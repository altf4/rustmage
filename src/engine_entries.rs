//! Owned pre-entry attachment/life choices; no object moves until the batch is ready.
use super::entry_tap::{EntryTapEffect, EntryTapEvent, EntryTapKind};
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryMove {
    pub delayed_return_exile: Option<Box<SpellView>>,
    pub choice: Option<EntryChoiceValue>,
    pub gift: Option<Box<GiftContext>>,
    pub before: ObjectView,
    /// Previously chosen zone replacement for an atomic cost transaction.
    pub zone_plan: Option<zone_replacements::Plan>,
    /// Original source-zone view when this entry is projected using another face.
    pub source_before: Option<Box<ObjectView>>,
    /// Physical card hidden behind face-down entry characteristics.
    pub manifest_original: Option<Card>,
    pub manifest_known: bool,
    pub controller: Player,
    /// Protector selected as this Battle enters, before replacement effects.
    pub battle_protector: Option<Player>,
    pub destination: Zone,
    pub tapped: bool,
    pub attachment: Option<ObjectRef>,
    /// Owned completion tied to this move's actual successful public result.
    pub role_after: Option<(Card, Player)>,
    pub suspect_after: bool,
    /// A surrounding instruction observes only this actual successful entry.
    pub life_bonus: Option<(Player, &'static str, u16)>,
    /// Counters put on this object as part of entering by the surrounding effect.
    pub effect_counter: Option<(CounterKind, u64)>,
    /// Timestamped ability grant after only this move actually enters.
    pub keywords_after: Option<(ObjectRef, Keywords)>,
    /// Only this surrounding return effect captures its actual public entry result.
    pub equipment_after: bool,
    /// Applied after entry observers freeze their facts, only on successful entry.
    pub blank_with_haste_after: Option<ObjectRef>,
    /// After this successful entry event, attach this Equipment to an exact recipient.
    pub attach_equipment_to: Option<ObjectRef>,
    pub return_modification: Option<(ReturnKind, ObjectRef)>,
    pub ghost_vacuum_after: Option<ObjectRef>,
    /// Source-independent type replacement and Food activation as this card enters.
    pub food_return: Option<Card>,
    /// Set artifact card type as it enters, preserving abilities (CR205.1a/611.2e).
    pub artifact_return: bool,
    pub attacking: Option<CombatDefender>,
    pub skipped: bool,
    pub reveal_hand: bool,
    /// A played library land is public throughout its pre-entry special action.
    /// Its actual origin remains Library; unrelated library entries stay private.
    pub public_origin: bool,
    pub life_paid: Option<bool>,
    pub offspring_paid: bool,
    pub cast_payment: Option<Mana>,
    pub cast_event: Option<Box<CastPaymentEvent>>,
    /// Explicit origin of an actual spell cast; absent for reanimation/put effects.
    pub cast_origin: Option<Zone>,
    pub warp_controller: Option<Player>,
    pub kicked: bool,
    pub bargained: bool,
    pub x: u32,
    pub evidence_collected: bool,
    /// Frozen before replacement choices; payments in this event cannot enable it.
    pub counters: Counters,
}
/// Owned prepayment state and explicit unpaid-program continuation. The
/// checkpoint is captured before zone replacement planning starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AtomicCostCheckpoint {
    pub before: Box<State>,
    pub bytes: usize,
    pub failure_pc: Option<usize>,
    pub components: Vec<(ObjectRef, Zone)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryBatch {
    pub atomic_cost: Option<Box<AtomicCostCheckpoint>>,
    pub temporary_copy: Option<Box<SpellView>>,
    pub discard_context: Option<discard_events::DiscardContext>,
    pub moves: Vec<EntryMove>,
    pub choices: Vec<usize>,
    pub next: usize,
    pub counter_next: usize,
    pub entry_bonuses_ready: bool,
    pub counter_order: Vec<usize>,
    pub timestamp_order: Vec<usize>,
    pub timestamps_ready: bool,
    pub resume: Option<Pending>,
    pub shuffle_after: Option<Player>,
    /// A surrounding effect waits for this immediate entry before its next event.
    pub replacement_resume: Option<Box<replacements::ReplacementState>>,
    /// Allocated only for events with competing untapped-entry replacements.
    pub tap_events: Vec<EntryTapEvent>,
    pub tap_next: usize,
}
impl EntryMove {
    pub(super) fn choose_projected_battle_protector(&mut self, kinds: u16, siege: bool) {
        self.battle_protector =
            if self.destination == Zone::Battlefield && kinds & crate::cards::BATTLE_KIND != 0 {
                Some(if siege {
                    self.controller.other()
                } else {
                    self.controller
                })
            } else {
                None
            };
    }
    pub(super) fn choose_battle_protector(&mut self) {
        self.choose_projected_battle_protector(
            self.entry_kinds(),
            Game::entry_choice_subtypes(self).has(self.before.card, "Siege"),
        );
    }
    pub(super) fn entry_kinds(&self) -> u16 {
        if self.food_return.is_some() {
            16
        } else if self.artifact_return {
            16 | (self.before.kinds & (4 | 8))
        } else {
            self.before.kinds
        }
    }
    pub(super) fn from_view(before: ObjectView, destination: Zone, tapped: bool) -> Self {
        EntryMove {
            delayed_return_exile: None,
            choice: None,
            gift: None,
            manifest_original: None,
            manifest_known: false,
            controller: before.controller,
            battle_protector: None,
            before,
            source_before: None,
            zone_plan: None,
            destination,
            tapped,
            attachment: None,
            role_after: None,
            suspect_after: false,
            life_bonus: None,
            effect_counter: None,
            keywords_after: None,
            equipment_after: false,
            blank_with_haste_after: None,
            attach_equipment_to: None,
            return_modification: None,
            ghost_vacuum_after: None,
            food_return: None,
            artifact_return: false,
            attacking: None,
            skipped: false,
            reveal_hand: false,
            public_origin: false,
            life_paid: None,
            offspring_paid: false,
            cast_payment: None,
            cast_event: None,
            cast_origin: None,
            warp_controller: None,
            kicked: false,
            bargained: false,
            x: 0,
            evidence_collected: false,
            counters: Counters::default(),
        }
    }
}
impl EntryBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.atomic_cost.as_ref().map_or(0, |a| {
                std::mem::size_of::<AtomicCostCheckpoint>()
                    + a.bytes
                    + a.components.len() * std::mem::size_of::<(ObjectRef, Zone)>()
            })
            + self.temporary_copy.as_ref().map_or(0, |item| {
                std::mem::size_of::<SpellView>() + item.owned_bytes()
            })
            + self.discard_context.as_ref().map_or(0, |c| c.bytes())
            + self.moves.len() * std::mem::size_of::<EntryMove>()
            + self
                .moves
                .iter()
                .map(|m| {
                    m.delayed_return_exile.as_ref().map_or(0, |item| {
                        std::mem::size_of::<SpellView>() + item.owned_bytes()
                    }) + m.counters.owned_bytes()
                        + m.before.counters.owned_bytes()
                        + m.before.temporary_copy_bytes()
                        + m.source_before.as_ref().map_or(0, |v| {
                            std::mem::size_of::<ObjectView>()
                                + v.counters.owned_bytes()
                                + v.temporary_copy_bytes()
                        })
                })
                .sum::<usize>()
            + self
                .tap_events
                .iter()
                .map(EntryTapEvent::bytes)
                .sum::<usize>()
            + self.moves.iter().filter(|m| m.cast_event.is_some()).count()
                * std::mem::size_of::<CastPaymentEvent>()
            + self.moves.iter().filter(|m| m.gift.is_some()).count()
                * std::mem::size_of::<GiftContext>()
            + self.choices.len() * std::mem::size_of::<usize>()
            + self.counter_order.len() * std::mem::size_of::<usize>()
            + self.timestamp_order.len() * std::mem::size_of::<usize>()
            + self.resume.as_ref().map_or(0, Pending::owned_bytes)
            + self.replacement_resume.as_ref().map_or(0, |f| f.bytes())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct AuraAttachmentView {
    pub aura: ObjectRef,
    pub card: Card,
    pub attachment: ObjectRef,
}
#[derive(Clone, Debug, Serialize)]
pub struct AuraEntryView {
    pub aura: ObjectRef,
    pub card: Card,
    pub candidates: Vec<ObjectRef>,
    pub selected: Vec<AuraAttachmentView>,
    pub selected_life: Vec<EntryLifeSelection>,
}
#[derive(Clone, Debug, Serialize)]
pub struct EntryLifeSelection {
    pub object: ObjectRef,
    pub card: Card,
    pub cost: u32,
    pub paid: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct EntryLifeView {
    pub object: ObjectRef,
    pub card: Card,
    pub cost: u32,
    /// Another effect already requires tapped entry. Paying does not undo it.
    pub already_tapped: bool,
    pub selected: Vec<EntryLifeSelection>,
    pub selected_attachments: Vec<AuraAttachmentView>,
}
pub(super) fn visible(m: &EntryMove, viewer: Player, known_hand: &[ObjectRef]) -> bool {
    visible_mask(m, 1 << viewer.index(), known_hand)
}
pub(super) fn visible_mask(m: &EntryMove, viewers: u8, known_hand: &[ObjectRef]) -> bool {
    m.public_origin
        || viewers & (1 << m.before.owner.index()) != 0
        || known_hand.contains(&m.before.reference)
        || !matches!(m.before.zone, Zone::Hand | Zone::Library)
}
impl EntryBatch {
    fn selected_life(&self, viewers: u8, known_hand: &[ObjectRef]) -> Vec<EntryLifeSelection> {
        self.moves
            .iter()
            .filter(|m| visible_mask(m, viewers, known_hand))
            .filter_map(|m| {
                let EntryTapped::PayLife(cost) = m.before.card.entry_tapped() else {
                    return None;
                };
                Some(EntryLifeSelection {
                    object: m.before.reference,
                    card: m.before.card,
                    cost,
                    paid: m.life_paid?,
                })
            })
            .collect()
    }
    fn selected_attachments(
        &self,
        viewers: u8,
        known_hand: &[ObjectRef],
    ) -> Vec<AuraAttachmentView> {
        self.moves
            .iter()
            .filter(|m| visible_mask(m, viewers, known_hand))
            .filter_map(|m| {
                Some(AuraAttachmentView {
                    aura: m.before.reference,
                    card: m.before.card,
                    attachment: m.attachment?,
                })
            })
            .collect()
    }
}
impl Game {
    pub(super) fn aura_can_enchant(
        &self,
        aura: Card,
        controller: Player,
        target: ObjectRef,
    ) -> bool {
        let Some(spec) = aura.aura() else {
            return false;
        };
        self.on_battlefield(target.card)
            && self.object_ref(target.card) == target
            && !self
                .keywords(target.card)
                .contains(Keywords::PROTECTION_EVERYTHING)
            && (if spec.kinds == 0 {
                self.current_kinds(target.card) & (1 | 2 | 16 | 32 | 64 | 256) != 0
            } else {
                self.current_kinds(target.card) & u16::from(spec.kinds) != 0
            })
            && ability_world::aura_artifact_or_non_aura_enchantment(self, aura, target.card)
            && match spec.relation {
                0 => self.state.objects[target.card as usize].controller == controller,
                1 => self.state.objects[target.card as usize].controller != controller,
                _ => true,
            }
    }
    pub(super) fn entry_move(&self, id: CardId, destination: Zone, tapped: bool) -> EntryMove {
        EntryMove::from_view(self.object_view(id), destination, tapped)
    }
    pub(super) fn stage_entries(&mut self, moves: Vec<EntryMove>, shuffle_after: Option<Player>) {
        self.stage_entries_inner(moves, shuffle_after, None, None, None);
    }
    pub(super) fn stage_discard_entries(
        &mut self,
        moves: Vec<EntryMove>,
        context: discard_events::DiscardContext,
    ) {
        self.stage_entries_inner(moves, None, Some(context), None, None);
    }
    pub(super) fn stage_temporary_copy_entries(&mut self, moves: Vec<EntryMove>, item: &SpellView) {
        self.stage_entries_inner(moves, None, None, Some(Box::new(item.clone())), None);
    }
    pub(super) fn stage_atomic_cost_entries(
        &mut self,
        moves: Vec<EntryMove>,
        checkpoint: AtomicCostCheckpoint,
    ) {
        self.stage_entries_inner(moves, None, None, None, Some(Box::new(checkpoint)));
    }
    fn stage_entries_inner(
        &mut self,
        mut moves: Vec<EntryMove>,
        shuffle_after: Option<Player>,
        discard_context: Option<discard_events::DiscardContext>,
        temporary_copy: Option<Box<SpellView>>,
        atomic_cost: Option<Box<AtomicCostCheckpoint>>,
    ) {
        if atomic_cost.is_none() && self.replay_frozen_mana_entries() {
            return;
        }
        assert!(
            self.state.entry_batch.is_none(),
            "one entry transaction at a time"
        );
        let mut tap_events = Vec::new();
        for (index, movement) in moves.iter_mut().enumerate() {
            movement.choose_battle_protector();
            if movement.before.kinds & crate::cards::BATTLE_KIND != 0
                && let Some(view) = self.preview_battle_entry(movement)
            {
                movement.choose_projected_battle_protector(view.kinds, view.has_subtype("Siege"));
            }
            if movement.destination != Zone::Battlefield {
                continue;
            }
            let intrinsic_tapped = if movement.manifest_original.is_some() {
                false
            } else {
                match movement.before.card.entry_tapped() {
                    EntryTapped::ChosenXAtMost(maximum) => movement.x <= maximum,
                    _ => self.self_entry_requires_tapped(movement.before.id, movement.controller),
                }
            };
            let mut effects = self.entry_tap_effects(movement);
            if movement.manifest_original.is_none()
                && let EntryTapped::UnlessBehold(subtype) = movement.before.card.entry_tapped()
            {
                effects.push(EntryTapEffect {
                    source: movement.before.reference,
                    intrinsic: true,
                    kind: EntryTapKind::Behold(subtype),
                });
            }
            if effects.is_empty() {
                movement.tapped |= intrinsic_tapped
                    || self.opponent_entry_kinds_require_tapped(
                        if movement.ghost_vacuum_after.is_some() {
                            self.ghost_entry_preview(movement)
                                .map_or_else(|| movement.entry_kinds(), |view| view.kinds)
                        } else {
                            movement.entry_kinds()
                        },
                        movement.controller,
                    );
            } else {
                // The incoming event retains instruction-level tapped entry;
                // intrinsic/global tapped replacements have not yet applied.
                if intrinsic_tapped {
                    effects.push(EntryTapEffect {
                        source: movement.before.reference,
                        intrinsic: true,
                        kind: EntryTapKind::Tapped,
                    });
                }
                if let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() {
                    effects.push(EntryTapEffect {
                        source: movement.before.reference,
                        intrinsic: true,
                        kind: EntryTapKind::PayLife(cost),
                    });
                }
                tap_events.push(EntryTapEvent {
                    index,
                    effects,
                    payment: None,
                    behold: None,
                });
            }
        }
        tap_events.sort_by_key(|event| {
            (
                moves[event.index].controller != self.state.active,
                event.index,
            )
        });
        // Entry replacements inspect the pre-event world, before any life-payment
        // choices. Later ETB triggers will instead observe the committed payments.
        for movement in &mut moves {
            movement.counters = self.pre_entry_counters_with_payment(
                &movement.before,
                movement.destination,
                movement.cast_payment,
                movement.controller,
                movement.x,
            );
            if movement.destination == Zone::Battlefield && movement.battle_protector.is_none() {
                movement
                    .counters
                    .remove(CounterKind::Defense, movement.before.card.printed_defense());
            }
            self.apply_cast_entry_counters(movement);
            if movement.destination == Zone::Battlefield && movement.cast_origin.is_some() {
                for &(kind, amount) in movement.before.card.cast_entry_counters() {
                    movement.counters.add(kind, amount);
                }
            }
            if movement.destination == Zone::Battlefield && movement.cast_origin == Some(Zone::Hand)
            {
                for &(kind, amount) in movement.before.card.hand_cast_entry_counters() {
                    movement.counters.add(kind, amount);
                }
            }
            if movement.destination == Zone::Battlefield
                && let Some((kind, amount)) = movement.effect_counter
            {
                movement.counters.add(kind, amount);
            }
            if movement.before.card.x_entry_counters() && movement.destination == Zone::Battlefield
            {
                movement
                    .counters
                    .add(CounterKind::PlusOne, u64::from(movement.x));
            }
            if movement.kicked && movement.destination == Zone::Battlefield {
                movement.counters.add(
                    CounterKind::PlusOne,
                    u64::from(movement.before.card.kicked_entry_counters()),
                );
            }
        }
        let mut choices: Vec<_> = moves
            .iter()
            .enumerate()
            .filter(|(i, m)| {
                m.destination == Zone::Battlefield
                    && (m.before.card.entry_choice().is_some()
                        || !m.artifact_return
                            && m.before.card.aura().is_some()
                            && m.attachment.is_none()
                        || matches!(m.before.card.entry_tapped(), EntryTapped::PayLife(_))
                            && m.life_paid.is_none()
                            && !tap_events.iter().any(|event| event.index == *i))
            })
            .map(|(i, _)| i)
            .collect();
        choices.sort_by_key(|&i| (moves[i].controller != self.state.active, i));
        // Counter replacement choices are APNAP independently of the move order.
        // Preserve moves and existing Aura/life indices for the simultaneous commit.
        let mut counter_order = moves
            .iter()
            .enumerate()
            .filter(|(_, m)| m.destination == Zone::Battlefield && m.counters.has_any())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        counter_order.sort_by_key(|&i| (moves[i].controller != self.state.active, i));
        let mut batch = EntryBatch {
            atomic_cost,
            temporary_copy,
            discard_context,
            moves,
            choices,
            next: 0,
            counter_next: 0,
            entry_bonuses_ready: false,
            counter_order,
            timestamp_order: Vec::new(),
            timestamps_ready: false,
            resume: None,
            shuffle_after,
            replacement_resume: None,
            tap_events,
            tap_next: 0,
        };
        if batch.choices.is_empty() {
            self.add_subtype_entry_counters(&mut batch.moves);
            batch.entry_bonuses_ready = true;
            Self::rebuild_entry_counter_order(&mut batch, self.state.active);
        }
        let counter_replacements = batch.moves.iter().any(|m| {
            m.destination == Zone::Battlefield
                && m.counters.has_any()
                && (!m.before.card.replacement_effects().is_empty()
                    || self.state.static_sources.iter().any(|&id| {
                        self.state.objects[id as usize]
                            .card
                            .replacement_effects()
                            .iter()
                            .any(|e| {
                                matches!(
                                    e,
                                    crate::cards::ReplacementSpec::Counters { .. }
                                        | crate::cards::ReplacementSpec::CountersPlacedByController
                                )
                            })
                    }))
        });
        // Atomic costs must always pass through advance_entries, even when
        // there are no entry choices: that path revalidates every component
        // and rolls the entire payment back before any event is committed.
        if batch.atomic_cost.is_none()
            && batch.choices.is_empty()
            && !batch
                .moves
                .iter()
                .any(|movement| movement.manifest_original.is_some())
            && batch.tap_events.is_empty()
            && !counter_replacements
            && !self.entry_needs_timestamp_choice(&batch)
        {
            self.commit_entries(batch);
        } else {
            self.state.entry_batch = Some(Box::new(batch));
        }
    }
    pub(super) fn has_entry_counter_replacements(&self, card: Card) -> bool {
        self.has_subtype_entry_counter_provider()
            || (card.enters_counters().total() > 0
                || !card.entry_counter_rules().is_empty()
                || card.x_complement_entry_counters().is_some())
                && (card.replacement_effects().iter().any(|e| {
                    matches!(
                        e,
                        crate::cards::ReplacementSpec::Counters { .. }
                            | crate::cards::ReplacementSpec::CountersPlacedByController
                    )
                }) || self.state.static_sources.iter().any(|&id| {
                    self.state.objects[id as usize]
                        .card
                        .replacement_effects()
                        .iter()
                        .any(|e| {
                            matches!(
                                e,
                                crate::cards::ReplacementSpec::Counters { .. }
                                    | crate::cards::ReplacementSpec::CountersPlacedByController
                            )
                        })
                }))
    }
    pub(super) fn pre_entry_counters(&self, before: &ObjectView, destination: Zone) -> Counters {
        self.pre_entry_counters_with_payment(before, destination, None, before.controller, 0)
    }
    fn pre_entry_counters_with_payment(
        &self,
        before: &ObjectView,
        destination: Zone,
        payment: Option<Mana>,
        controller: Player,
        x: u32,
    ) -> Counters {
        if destination != Zone::Battlefield {
            return Counters::default();
        }
        let mut counters = before.card.enters_counters();
        if let Some((kind, maximum)) = before.card.x_complement_entry_counters() {
            counters.add(kind, u64::from(maximum.saturating_sub(x)));
        }
        for rule in before.card.entry_counter_rules() {
            if self.step_guard(rule.guard, controller, before.reference, before.tapped) {
                let amount = match rule.amount {
                    EntryCounterAmount::ControlledCreatureDeaths => {
                        self.state.players[controller.index()].controlled_creatures_died_this_turn
                    }
                    EntryCounterAmount::GreatestOtherCreaturePower => self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == controller
                                && *id as CardId != before.id
                                && self.current_kinds(*id as CardId) & 1 != 0
                        })
                        .map(|(id, _)| self.stats(id as CardId).power)
                        .max()
                        .unwrap_or(0)
                        .max(0)
                        .try_into()
                        .capacity("entry power exhausted"),
                    EntryCounterAmount::ColorsSpent => {
                        payment.map_or(0, |p| (0..5).filter(|&i| p[i] > 0).count() as u64)
                    }
                    EntryCounterAmount::IfNotCastOrZero(n) => {
                        if payment.is_none_or(|p| p == [0; 6]) {
                            u64::from(n)
                        } else {
                            0
                        }
                    }
                    EntryCounterAmount::Fixed(n) => u64::from(n),
                    EntryCounterAmount::Stat(value) => {
                        u64::try_from(self.stat_value(value, before.reference, controller).max(0))
                            .capacity("entry counter amount exhausted")
                    }
                };
                counters.add(CounterKind::PlusOne, amount);
            }
        }
        counters
    }
    pub(super) fn commit_entries(&mut self, mut batch: EntryBatch) {
        for movement in &batch.moves {
            if movement.skipped
                && movement.before.zone == Zone::Outside
                && movement.before.is_token()
            {
                self.release_unentered_copy(movement.before.reference);
            }
        }
        let mut timestamp_order = batch.timestamp_order.clone();
        if !batch.timestamps_ready {
            // No group has multiple objects, so APNAP determines the order uniquely.
            assert!(!self.entry_needs_timestamp_choice(&batch));
            timestamp_order = self.entry_timestamp_indices(&batch);
        }
        batch.timestamp_order = timestamp_order.clone();
        batch.timestamps_ready = true;
        self.record_cast_mana_entries(&batch);
        let hand_cards: Vec<_> = Self::hand_entry_order(&batch, self.state.active)
            .iter()
            .map(|&i| batch.moves[i].before.id)
            .collect();
        if !hand_cards.is_empty() {
            self.state.hand_size.get_or_insert_default().deferred = true;
        }
        let timestamp_cards: Vec<_> = timestamp_order
            .iter()
            .filter(|&&i| Self::entry_has_timestamp(&batch.moves[i]))
            .map(|&i| batch.moves[i].before.id)
            .collect();
        if !timestamp_cards.is_empty() {
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .deferred_timestamps = true;
        }
        // Apply quantities captured before this event's replacement choices.
        // No incoming permanent or same-event life payment changes those values.
        let equipment_origin = batch
            .moves
            .iter()
            .find(|m| m.equipment_after)
            .map(|m| m.before.reference);
        let moves: Vec<_> = batch
            .moves
            .into_iter()
            .filter(|m| !m.skipped)
            .map(|m| {
                // Impossible Aura entry can redirect an already staged move.
                let counters = if m.destination == Zone::Battlefield {
                    m.counters.clone()
                } else {
                    Counters::default()
                };
                (m, counters)
            })
            .collect();
        let returned_hand_observers = if moves
            .iter()
            .any(|(m, _)| m.before.zone == Zone::Battlefield && m.destination == Zone::Hand)
        {
            self.freeze_returned_hand_observers()
        } else {
            None
        };
        let graveyard_observers = if moves.iter().any(|(m, _)| m.before.zone == Zone::Graveyard) {
            self.freeze_graveyard_observers()
        } else {
            None
        };
        self.capture_event_object_facts(moves.iter().map(|(movement, _)| {
            movement
                .source_before
                .as_deref()
                .unwrap_or(&movement.before)
        }));
        self.capture_source_quantities(moves.iter().map(|(movement, _)| {
            movement
                .source_before
                .as_deref()
                .unwrap_or(&movement.before)
        }));
        let mut events = Vec::new();
        let mut attachments = Vec::new();
        let mut role_results = Vec::new();
        let mut suspect_results = Vec::new();
        let mut life_results = Vec::new();
        let mut equipment_results = Vec::new();
        let mut keyword_results = Vec::new();
        let mut ability_results = Vec::new();
        let mut food_results = Vec::new();
        let mut artifact_results = Vec::new();
        let mut ghost_results = Vec::new();
        for (movement, counters) in moves {
            let entering_face = movement
                .source_before
                .as_ref()
                .map(|_| movement.before.card);
            let before_battle = movement.before.kinds & crate::cards::BATTLE_KIND != 0;
            let manifest_face = movement.before.card;
            let mut before = movement.source_before.map_or(movement.before, |v| *v);
            let plan = movement
                .zone_plan
                .clone()
                .filter(|plan| plan.destination == movement.destination)
                .unwrap_or_else(|| {
                    let rules = self.freeze_zone_rules(&before, movement.destination, None);
                    self.plan_zone_move(&before, movement.destination, rules)
                });
            if movement.manifest_original.is_some() {
                if plan.destination == Zone::Battlefield {
                    self.state.objects[before.id as usize].card = manifest_face;
                } else {
                    before = self.object_view(before.id);
                }
            }
            let mut event = self.commit_planned_zone_move(
                before,
                plan,
                movement.tapped,
                counters,
                EntryIdentity {
                    controller: Some(movement.controller),
                    face: entering_face,
                },
                ZoneObservers {
                    graveyard: graveyard_observers.clone(),
                    returned_hand: returned_hand_observers.clone(),
                },
            );
            if event.moved && event.destination == Zone::Battlefield {
                if let Some(item) = &movement.delayed_return_exile {
                    self.register_delayed_return_exile(item, event.after);
                }
                if let Some(protector) = movement.battle_protector {
                    self.set_battle_protector(event.after, protector);
                } else if before_battle {
                    self.clear_battle_protector(event.after);
                }
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && let Some(original) = movement.manifest_original
            {
                let known_to = self.face_down_knowledge_mask(movement.controller);
                self.state
                    .manifest
                    .get_or_insert_with(Default::default)
                    .permanents
                    .push(manifest::FaceDownPermanent {
                        object: event.after,
                        underlying: original,
                        known: movement.manifest_known,
                        known_to,
                    });
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && let Some(value) = movement.choice
            {
                self.set_entry_value(event.after, value);
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && let Some((kind, source)) = movement.return_modification
            {
                self.apply_return_modification(event.after, kind, source);
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && let Some(definition) = movement.food_return
            {
                food_results.push((event.after, definition));
            }
            if event.moved && event.destination == Zone::Battlefield && movement.artifact_return {
                artifact_results.push(event.after);
            }
            if event.moved
                && event.destination == Zone::Battlefield
                && let Some(source) = movement.ghost_vacuum_after
            {
                ghost_results.push((event.after, source));
            }
            event.cast_event = movement.cast_event;
            event.was_cast = movement.cast_origin.is_some();
            event.offspring_paid = movement.offspring_paid;
            event.kicked = movement.kicked;
            if event.moved && event.destination == Zone::Battlefield {
                event.gift = movement.gift.clone();
            }
            event.bargained = movement.bargained;
            event.x = movement.x;
            event.evidence_collected = movement.evidence_collected;
            if event.moved {
                if event.destination == Zone::Battlefield
                    && let Some((source, keywords)) = movement.keywords_after
                {
                    keyword_results.push((event.after, source, keywords));
                }
                if event.destination == Zone::Battlefield
                    && let Some(source) = movement.blank_with_haste_after
                {
                    ability_results.push((event.after, source));
                }
                if event.destination == Zone::Battlefield
                    && let Some(recipient) = movement.attach_equipment_to
                {
                    equipment_results.push((event.after, recipient));
                }
                if event.destination == Zone::Battlefield
                    && let Some((controller, subtype, amount)) = movement.life_bonus
                {
                    life_results.push((event.after, controller, subtype, amount));
                }
                if event.destination == Zone::Battlefield && movement.suspect_after {
                    suspect_results.push(event.after);
                }
                if event.destination == Zone::Battlefield
                    && let Some((token, creator)) = movement.role_after
                {
                    role_results.push((creator, token, event.after));
                }
                if event.destination == Zone::Battlefield
                    && let Some(defender) = movement.attacking
                    && self.is_creature(event.after.card)
                    && self.current_kinds(event.after.card) & crate::cards::BATTLE_KIND == 0
                    && self.attacking_return_combat(movement.controller)
                    && match defender {
                        CombatDefender::Player(p) => p == movement.controller.other(),
                        CombatDefender::Battle(r) => {
                            self.battle_can_be_attacked_by(r, movement.controller)
                        }
                        CombatDefender::Planeswalker(r) => {
                            self.on_battlefield(r.card)
                                && self.object_ref(r.card) == r
                                && self.state.objects[r.card as usize].controller
                                    == movement.controller.other()
                                && self.current_kinds(r.card) & 64 != 0
                        }
                    }
                {
                    self.state.attackers.push(Attacker {
                        card: event.after.card,
                        defender: Some(defender),
                        blocked: false,
                    });
                }

                if let Some(target) = movement.attachment {
                    attachments.push((event.after, target));
                }
                if movement.reveal_hand && event.destination == Zone::Hand {
                    self.state.known_hand.push(event.after);
                }
            }
            if let Some(controller) = movement.warp_controller
                && event.moved
                && event.destination == Zone::Battlefield
            {
                self.state.objects[event.after.card as usize].cast_for_warp = event.was_cast;
                self.register_warp_exile(&event, controller);
            }
            events.push(event);
        }
        // Continuous attachment effects already exist when entry events inspect the world.
        for (source, target) in attachments {
            self.state.objects[source.card as usize].attached_to = Some(target);
            self.retimestamp_live_static_source(source);
            if self.state.objects[source.card as usize]
                .card
                .role()
                .is_some()
            {
                let timestamp = self.next_role_timestamp();
                self.record_role(source, target, timestamp);
            }
        }
        if !timestamp_cards.is_empty() {
            self.state.roles.as_mut().unwrap().deferred_timestamps = false;
            for id in timestamp_cards {
                if let Some(event) = events
                    .iter()
                    .find(|e| e.before.id == id && e.moved && e.destination == Zone::Battlefield)
                {
                    self.assign_static_source_timestamp(event.after);
                }
            }
        }
        if !hand_cards.is_empty() {
            self.state.hand_size.as_mut().unwrap().deferred = false;
            for id in hand_cards {
                if let Some(event) = events
                    .iter()
                    .find(|e| e.before.id == id && e.moved && e.destination == Zone::Battlefield)
                {
                    self.retimestamp_hand_size_source(event.after);
                }
            }
        }
        if !ghost_results.is_empty() {
            let timestamp = self.next_role_timestamp();
            let roles = self.state.roles.get_or_insert_with(Default::default);
            for (recipient, source) in ghost_results {
                roles
                    .creature_types
                    .push(subtypes::CreatureTypesEffectView {
                        recipient,
                        source,
                        types: &["Spirit"],
                        additional: true,
                        timestamp,
                    });
                roles.base_stats.push(base_stats::BaseStatsEffectView {
                    recipient,
                    source,
                    power: Some(1),
                    toughness: Some(1),
                    timestamp,
                    static_source: false,
                    permanent: true,
                });
            }
        }
        // CR613.7n: entrant own statics receive earlier relative timestamps
        // than the surrounding as-entry continuous effect. All forms exist
        // before the one simultaneous event batch freezes entry observers.
        for (recipient, definition) in food_results {
            self.apply_food_return(recipient, definition);
        }
        for recipient in artifact_results {
            let timestamp = self.next_role_timestamp();
            self.state.roles.get_or_insert_default().animations.push(
                characteristic_layers::artifact_noncreature_form(recipient, timestamp),
            );
        }
        self.refresh_controllers();
        if let Some(item) = &batch.temporary_copy {
            let tokens = events
                .iter()
                .filter(|e| e.moved && e.destination == Zone::Battlefield)
                .map(|e| e.after)
                .collect::<Vec<_>>();
            if !tokens.is_empty() {
                self.finish_temporary_copy(item, tokens);
            }
        }
        if let Some(origin) = equipment_origin {
            self.capture_return_attachment_entry(origin, &events);
        }
        self.capture_selected_library_entry(&events);
        self.capture_search_result_entry(&events);
        if let Some(context) = batch.discard_context {
            self.finish_discard_events(&events, context);
        } else {
            self.finish_zone_events(&events, &[]);
        }
        // The entry event and its frozen trigger facts precede this subsequent
        // attachment. Packets cannot be stacked until the surrounding resolution ends.
        for (recipient, source, keywords) in keyword_results {
            if self.object_ref(recipient.card) == recipient && self.on_battlefield(recipient.card) {
                self.grant_keyword_effects(&[recipient.card], source, keywords);
            }
        }
        for (recipient, source) in ability_results {
            self.blank_and_grant_haste(recipient, source);
        }
        for (equipment, recipient) in equipment_results {
            self.attach_equipment(equipment, recipient);
        }
        self.refresh_controllers();
        for (reference, controller, subtype, amount) in life_results {
            if self.object_ref(reference.card) == reference
                && self.on_battlefield(reference.card)
                && self.has_subtype(reference.card, subtype)
            {
                self.gain_life(controller, i128::from(amount));
            }
        }
        // The entry event precedes the subsequent suspect instruction.
        for recipient in suspect_results {
            self.set_suspected(recipient, true);
        }
        for (controller, token, recipient) in role_results {
            if self.aura_can_enchant(token, controller, recipient) {
                self.stage_token_creation(
                    controller,
                    token,
                    1,
                    false,
                    replacements::TokenCompletion::RoleAttach(recipient),
                );
            }
        }
        if let Some(player) = batch.shuffle_after {
            if self.state.players[player.index()].library.len() > 1 {
                self.forget_library_top(player);
            }
            self.shuffle_library_cards(player);
        }
    }
    /// Settle transactions until an actual player choice blocks. Call at the outer
    /// action boundary, after the initiating cost/announcement has finished its bookkeeping.
    pub(super) fn advance_entries(&mut self) -> bool {
        loop {
            if self.state.outcome.is_some() {
                self.state.entry_batch = None;
                return false;
            }
            let Some(batch) = self.state.entry_batch.as_mut() else {
                return false;
            };
            if batch.resume.is_none() {
                batch.resume = Some(self.state.pending.clone());
            }
            if self.advance_entry_tap_effects() {
                return true;
            }
            while let Some((index, owner)) = self.state.entry_batch.as_ref().and_then(|batch| {
                batch
                    .choices
                    .get(batch.next)
                    .map(|&index| (index, batch.moves[index].controller))
            }) {
                let movement = &self.state.entry_batch.as_ref().unwrap().moves[index];
                if movement.before.card.entry_choice().is_some() && movement.choice.is_none() {
                    self.state.pending = Pending::EntryValue { player: owner };
                    return true;
                }
                if movement.life_paid.is_none()
                    && matches!(movement.before.card.entry_tapped(), EntryTapped::PayLife(_))
                {
                    if self.can_pay_entry_life() {
                        self.state.pending = Pending::EntryLife { player: owner };
                        return true;
                    }
                    // An optional positive payment is impossible below the cost.
                    self.choose_entry_life(false);
                    continue;
                }
                if movement.before.card.aura().is_none() || movement.attachment.is_some() {
                    self.state.entry_batch.as_mut().unwrap().next += 1;
                    continue;
                }
                let eligible = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .any(|(id, _)| self.entry_candidate(self.object_ref(id as CardId)));
                if eligible {
                    self.state.pending = Pending::AuraEntry { player: owner };
                    return true;
                }
                // No object moves for an impossible entry, except an Aura on the
                // stack goes to its owner's graveyard (CR303.4g).
                let batch = self.state.entry_batch.as_mut().unwrap();
                if batch.moves[index].before.zone == Zone::Stack {
                    batch.moves[index].destination = Zone::Graveyard;
                } else {
                    batch.moves[index].skipped = true;
                }
                batch.next += 1;
            }
            if !self.state.entry_batch.as_ref().unwrap().entry_bonuses_ready {
                let mut batch = self.state.entry_batch.take().unwrap();
                self.add_subtype_entry_counters(&mut batch.moves);
                batch.entry_bonuses_ready = true;
                Self::rebuild_entry_counter_order(&mut batch, self.state.active);
                self.state.entry_batch = Some(batch);
            }
            while self.state.entry_batch.as_ref().unwrap().counter_next
                < self.state.entry_batch.as_ref().unwrap().counter_order.len()
            {
                let batch = self.state.entry_batch.as_mut().unwrap();
                let index = batch.counter_order[batch.counter_next];
                batch.counter_next += 1;
                let m = batch.moves[index].clone();
                if m.destination != Zone::Battlefield || m.skipped || m.counters.is_empty() {
                    continue;
                }
                let p = replacements::CounterPlacement {
                    placed_by: m.controller,
                    object: m.before.reference,
                    card: m.before.card,
                    owner: m.before.owner,
                    controller: m.controller,
                    protector: m.battle_protector,
                    subtypes: if m.ghost_vacuum_after.is_some() {
                        self.ghost_entry_preview(&m).map_or_else(
                            || Self::entry_choice_subtypes(&m),
                            |view| view.granted_subtypes,
                        )
                    } else {
                        Self::entry_choice_subtypes(&m)
                    },
                    kind: if m.ghost_vacuum_after.is_some() {
                        self.ghost_entry_preview(&m)
                            .map_or_else(|| m.entry_kinds(), |view| view.kinds)
                    } else {
                        m.entry_kinds()
                    },
                    power: if m.ghost_vacuum_after.is_some()
                        || replacements::entry_power_sensitive(self, m.controller, m.before.card)
                    {
                        self.ghost_entry_preview(&m)
                            .map_or(m.before.power, |view| view.power)
                    } else {
                        m.before.power
                    },
                    amount: m.counters.clone(),
                    effect: true,
                    entry_index: Some(index),
                    private: !visible(&m, m.before.owner.other(), &self.state.known_hand),
                };
                self.stage_counter_placements(vec![p], replacements::ResumeMode::Entry);
                if self.advance_replacements(false) {
                    return true;
                }
            }
            if self.advance_entry_timestamps() {
                return true;
            }
            let mut batch = *self.state.entry_batch.take().unwrap();
            if let Some(checkpoint) = batch.atomic_cost.take() {
                let invalid = checkpoint.components.iter().any(|&(reference, zone)| {
                    self.state
                        .objects
                        .get(reference.card as usize)
                        .is_none_or(|object| {
                            object.generation != reference.generation || object.zone != zone
                        })
                });
                if invalid {
                    let mut before = *checkpoint.before;
                    zone_replacements::transfer_live_replay(
                        &mut self.state.zone_rules,
                        &mut before.zone_rules,
                    );
                    self.state = before;
                    if let Some(pc) = checkpoint.failure_pc {
                        self.state
                            .resolving
                            .as_mut()
                            .expect("atomic cost failure continuation")
                            .pc = pc;
                    }
                    if self.state.resolving.is_some() {
                        self.run_program();
                    }
                    return false;
                }
            }
            self.state.pending = batch.resume.clone().unwrap();
            let replacement_resume = batch.replacement_resume.take();
            let selected = self
                .state
                .manifest
                .as_ref()
                .and_then(|state| state.choice.as_ref())
                .and_then(|choice| choice.selected);
            let completes_manifest = batch.moves.iter().any(|movement| {
                movement.manifest_original.is_some() && Some(movement.before.reference) == selected
            });
            self.commit_entries(batch);
            if completes_manifest {
                self.finish_manifest_entry();
            }
            if let Some(outer) = replacement_resume {
                self.state.replacements = Some(outer);
                if self.advance_replacements(true) {
                    return true;
                }
            } else if self.state.resolving.is_some() {
                self.run_program();
            }
            if self.state.announcing.as_ref().is_some_and(|a| {
                a.additional_paid
                    || a.cast_costs
                        .as_ref()
                        .is_some_and(|c| c.mana_closed && c.active.is_none())
            }) {
                self.finish_announcement();
            }
        }
    }
    pub(super) fn entry_candidate(&self, target: ObjectRef) -> bool {
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[batch.choices[batch.next]];
        self.aura_can_enchant(movement.before.card, movement.controller, target)
            && batch.moves.iter().all(|m| m.before.id != target.card)
    }
    pub(super) fn choose_aura_attachment(&mut self, target: ObjectRef) {
        let batch = self.state.entry_batch.as_mut().unwrap();
        let index = batch.choices[batch.next];
        batch.moves[index].attachment = Some(target);
        batch.next += 1;
    }
    pub(super) fn current_entry_value_index(&self) -> Option<usize> {
        let batch = self.state.entry_batch.as_ref()?;
        if let Some(event) = batch.tap_events.get(batch.tap_next) {
            let movement = &batch.moves[event.index];
            if movement.choice.is_none()
                && movement.before.card.entry_choice() == Some(EntryChoiceKind::BasicLandType)
            {
                return Some(event.index);
            }
        }
        batch.choices.get(batch.next).copied()
    }
    pub(super) fn can_pay_entry_life(&self) -> bool {
        if let Some((movement, cost)) = self.entry_tap_payment() {
            return self.state.players[movement.controller.index()].life >= i128::from(cost);
        }
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[batch.choices[batch.next]];
        let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() else {
            unreachable!()
        };
        self.state.players[movement.controller.index()].life >= i128::from(cost)
    }
    pub(super) fn choose_entry_life(&mut self, pay: bool) {
        assert!(!pay || self.can_pay_entry_life());
        if self.finish_entry_tap_payment(pay) {
            return;
        }
        let batch = self.state.entry_batch.as_mut().unwrap();
        let movement = &mut batch.moves[batch.choices[batch.next]];
        let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() else {
            unreachable!()
        };
        let player = movement.controller;
        movement.life_paid = Some(pay);
        movement.tapped |= !pay;
        batch.next += 1;
        if pay {
            self.record_cast_mana_entry_life(player, cost);
            self.lose_life(player, i128::from(cost));
        }
    }
    pub(super) fn entry_life_view(&self, viewer: Player) -> Option<EntryLifeView> {
        if !matches!(self.state.pending, Pending::EntryLife { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = if let Some((movement, _)) = self.entry_tap_payment() {
            movement
        } else {
            batch.moves.get(*batch.choices.get(batch.next)?)?
        };
        if !visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() else {
            return None;
        };
        Some(EntryLifeView {
            object: movement.before.reference,
            card: movement.before.card,
            cost,
            already_tapped: movement.tapped,
            selected: batch.selected_life(self.information_mask(viewer), &self.state.known_hand),
            selected_attachments: batch
                .selected_attachments(self.information_mask(viewer), &self.state.known_hand),
        })
    }
    pub(super) fn aura_entry_view(&self, viewer: Player) -> Option<AuraEntryView> {
        if !matches!(self.state.pending, Pending::AuraEntry { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = if let Some((movement, _)) = self.entry_tap_payment() {
            movement
        } else {
            batch.moves.get(*batch.choices.get(batch.next)?)?
        };
        if !visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        Some(AuraEntryView {
            aura: movement.before.reference,
            card: movement.before.card,
            candidates: self
                .state
                .objects
                .iter()
                .enumerate()
                .map(|(i, _)| self.object_ref(i as CardId))
                .filter(|&r| self.entry_candidate(r))
                .collect(),
            selected: batch
                .selected_attachments(self.information_mask(viewer), &self.state.known_hand),
            selected_life: batch
                .selected_life(self.information_mask(viewer), &self.state.known_hand),
        })
    }
}

/// Public objects about to receive simultaneous timestamps, oldest first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryTimestampObjectView {
    pub object: ObjectRef,
    pub card: Card,
    pub controller: Player,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryTimestampView {
    pub controller: Player,
    pub candidates: Vec<EntryTimestampObjectView>,
    pub selected: Vec<EntryTimestampObjectView>,
}
impl Game {
    pub(super) fn entry_has_timestamp(m: &EntryMove) -> bool {
        m.destination == Zone::Battlefield
            && m.before.zone != Zone::Battlefield
            && (!m.before.is_token()
                || m.before.copy_recipe.is_some()
                || CounterAmounts::from(&m.counters)
                    .keyword_amounts()
                    .iter()
                    .any(|n| *n > 0))
            && !m.before.card.is_emblem()
            && !m.skipped
            && (m.artifact_return
                || m.ghost_vacuum_after.is_some()
                || m.food_return.is_some()
                || m.return_modification.is_some()
                || m.before.card.base_stat_setter().is_some()
                || !m.before.card.live_static_effects().is_empty()
                || m.before.card.role().is_some()
                || m.before.card.has_layer6_provider()
                || CounterAmounts::from(&m.counters)
                    .keyword_amounts()
                    .iter()
                    .any(|n| *n > 0))
    }
    fn entry_timestamp_indices(&self, batch: &EntryBatch) -> Vec<usize> {
        let mut ids: Vec<_> = batch
            .moves
            .iter()
            .enumerate()
            .filter_map(|(i, m)| {
                (Self::entry_has_timestamp(m)
                    || (batch.moves.iter().any(|m| {
                        Self::hand_entry(m) && m.before.card.hand_size_setting().flatten().is_some()
                    }) && Self::hand_entry(m)))
                .then_some(i)
            })
            .collect();
        ids.sort_by_key(|&i| batch.moves[i].controller != self.state.active);
        ids
    }
    fn entry_needs_timestamp_choice(&self, batch: &EntryBatch) -> bool {
        let indices = self.entry_timestamp_indices(batch);
        [Player::P0, Player::P1].iter().any(|&p| {
            indices
                .iter()
                .filter(|&&i| batch.moves[i].controller == p)
                .count()
                > 1
        })
    }
    fn remaining_timestamp_indices(&self) -> Vec<usize> {
        let batch = self.state.entry_batch.as_ref().unwrap();
        let ids: Vec<_> = self
            .entry_timestamp_indices(batch)
            .into_iter()
            .filter(|i| !batch.timestamp_order.contains(i))
            .collect();
        let Some(&first) = ids.first() else {
            return ids;
        };
        let controller = batch.moves[first].controller;
        ids.into_iter()
            .filter(|&i| batch.moves[i].controller == controller)
            .collect()
    }
    fn advance_entry_timestamps(&mut self) -> bool {
        loop {
            let ids = self.remaining_timestamp_indices();
            if ids.is_empty() {
                self.state.entry_batch.as_mut().unwrap().timestamps_ready = true;
                return false;
            }
            if ids.len() == 1 {
                self.state
                    .entry_batch
                    .as_mut()
                    .unwrap()
                    .timestamp_order
                    .push(ids[0]);
                continue;
            }
            let player = self.state.entry_batch.as_ref().unwrap().moves[ids[0]].controller;
            self.state.pending = Pending::EntryTimestamp { player };
            return true;
        }
    }
    pub(super) fn entry_timestamp_candidates(&self) -> Vec<ObjectRef> {
        let batch = self.state.entry_batch.as_ref().unwrap();
        self.remaining_timestamp_indices()
            .into_iter()
            .map(|i| batch.moves[i].before.reference)
            .collect()
    }
    pub(super) fn choose_entry_timestamp(&mut self, object: ObjectRef) {
        let i = self
            .remaining_timestamp_indices()
            .into_iter()
            .find(|&i| {
                self.state.entry_batch.as_ref().unwrap().moves[i]
                    .before
                    .reference
                    == object
            })
            .expect("validated timestamp choice");
        self.state
            .entry_batch
            .as_mut()
            .unwrap()
            .timestamp_order
            .push(i);
    }
    pub(super) fn entry_timestamp_view(&self, viewer: Player) -> Option<EntryTimestampView> {
        let Pending::EntryTimestamp { player } = self.state.pending else {
            return None;
        };
        let batch = self.state.entry_batch.as_ref()?;
        let view = |i: usize| {
            let m = &batch.moves[i];
            EntryTimestampObjectView {
                object: m.before.reference,
                card: m.before.card,
                controller: m.controller,
            }
        };
        // Entry remains a single transaction. Ordering continuous sources must
        // not publish cards that are still selected in a private library/hand.
        let visible_index = |&i: &usize| {
            visible_mask(
                &batch.moves[i],
                self.information_mask(viewer),
                &self.state.known_hand,
            )
        };
        let candidates: Vec<_> = self
            .remaining_timestamp_indices()
            .into_iter()
            .filter(visible_index)
            .map(view)
            .collect();
        let selected: Vec<_> = batch
            .timestamp_order
            .iter()
            .copied()
            .filter(visible_index)
            .map(view)
            .collect();
        if candidates.is_empty() && selected.is_empty() {
            return None;
        }
        Some(EntryTimestampView {
            controller: player,
            candidates,
            selected,
        })
    }
}

impl Game {
    fn has_subtype_entry_counter_provider(&self) -> bool {
        self.state.static_sources.iter().any(|&id| {
            self.state.objects[id as usize].zone == Zone::Battlefield
                && self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|f| {
                            matches!(
                                f,
                                crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(_)
                            )
                        })
                    })
        })
    }
    fn rebuild_entry_counter_order(batch: &mut EntryBatch, active: Player) {
        batch.counter_order = batch
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.destination == Zone::Battlefield && !m.skipped && m.counters.has_any()
            })
            .map(|(index, _)| index)
            .collect();
        batch
            .counter_order
            .sort_by_key(|&i| (batch.moves[i].controller != active, i));
    }
    fn add_subtype_entry_counters(&self, moves: &mut [EntryMove]) {
        // Only a live provider unlocks the battlefield count. Ordinary entry
        // batches never acquire an all-object scan from this mechanic.
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            if source.zone != Zone::Battlefield {
                continue;
            }
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    let crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(subtype) =
                        fragment
                    else {
                        continue;
                    };
                    if !self.continuous_fragment_available(id, |f| matches!(f,
                        crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(s) if s == subtype)) { continue }
                    let count = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(other, object)| {
                            object.zone == Zone::Battlefield
                                && object.controller == source.controller
                                && self
                                    .granted_subtypes(*other as CardId)
                                    .has(object.card, subtype)
                        })
                        .count();
                    for movement in moves.iter_mut() {
                        if movement.destination == Zone::Battlefield
                            && !movement.skipped
                            && movement.controller == source.controller
                            && movement.before.reference != self.object_ref(id)
                            && if movement.ghost_vacuum_after.is_some() {
                                self.ghost_entry_preview(movement)
                                    .is_some_and(|view| view.has_subtype(subtype))
                            } else {
                                Self::entry_choice_subtypes(movement)
                                    .has(movement.before.card, subtype)
                            }
                        {
                            movement.counters.add(
                                CounterKind::PlusOne,
                                u64::try_from(count).capacity("entry counter count exhausted"),
                            );
                        }
                    }
                }
            }
        }
    }
}
