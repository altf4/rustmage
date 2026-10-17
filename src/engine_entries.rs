//! Owned pre-entry attachment/life choices; no object moves until the batch is ready.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryMove {
    pub before: ObjectView,
    pub controller: Player,
    pub destination: Zone,
    pub tapped: bool,
    pub attachment: Option<ObjectRef>,
    /// Owned completion tied to this move's actual successful public result.
    pub role_after: Option<Card>,
    pub attacking: Option<CombatDefender>,
    pub skipped: bool,
    pub reveal_hand: bool,
    pub life_paid: Option<bool>,
    pub offspring_paid: bool,
    pub cast_payment: Option<Mana>,
    pub warp_controller: Option<Player>,
    pub kicked: bool,
    pub bargained: bool,
    pub x: u32,
    pub evidence_collected: bool,
    /// Frozen before replacement choices; payments in this event cannot enable it.
    pub counters: Counters,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryBatch {
    pub moves: Vec<EntryMove>,
    pub choices: Vec<usize>,
    pub next: usize,
    pub counter_next: usize,
    pub counter_order: Vec<usize>,
    pub resume: Option<Pending>,
    pub shuffle_after: Option<Player>,
    /// A surrounding effect waits for this immediate entry before its next event.
    pub replacement_resume: Option<Box<replacements::ReplacementState>>,
}
impl EntryBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.moves.len() * std::mem::size_of::<EntryMove>()
            + self.choices.len() * std::mem::size_of::<usize>()
            + self.counter_order.len() * std::mem::size_of::<usize>()
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
fn visible(m: &EntryMove, viewer: Player, known_hand: &[ObjectRef]) -> bool {
    m.before.owner == viewer
        || known_hand.contains(&m.before.reference)
        || !matches!(m.before.zone, Zone::Hand | Zone::Library)
}
impl EntryBatch {
    fn selected_life(&self, viewer: Player, known_hand: &[ObjectRef]) -> Vec<EntryLifeSelection> {
        self.moves
            .iter()
            .filter(|m| visible(m, viewer, known_hand))
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
        viewer: Player,
        known_hand: &[ObjectRef],
    ) -> Vec<AuraAttachmentView> {
        self.moves
            .iter()
            .filter(|m| visible(m, viewer, known_hand))
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
            && self.current_kinds(target.card) & u16::from(spec.kinds) != 0
            && match spec.relation {
                0 => self.state.objects[target.card as usize].controller == controller,
                1 => self.state.objects[target.card as usize].controller != controller,
                _ => true,
            }
    }
    pub(super) fn entry_move(&self, id: CardId, destination: Zone, tapped: bool) -> EntryMove {
        EntryMove {
            before: self.object_view(id),
            controller: self.state.objects[id as usize].controller,
            destination,
            tapped,
            attachment: None,
            role_after: None,
            attacking: None,
            skipped: false,
            reveal_hand: false,
            life_paid: None,
            offspring_paid: false,
            cast_payment: None,
            warp_controller: None,
            kicked: false,
            bargained: false,
            x: 0,
            evidence_collected: false,
            counters: Counters::default(),
        }
    }
    pub(super) fn stage_entries(
        &mut self,
        mut moves: Vec<EntryMove>,
        shuffle_after: Option<Player>,
    ) {
        assert!(
            self.state.entry_batch.is_none(),
            "one entry transaction at a time"
        );
        // Entry replacements inspect the pre-event world, before any life-payment
        // choices. Later ETB triggers will instead observe the committed payments.
        for movement in &mut moves {
            movement.counters = self.pre_entry_counters_with_payment(
                &movement.before,
                movement.destination,
                movement.cast_payment,
                movement.controller,
            );
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
            .filter(|(_, m)| {
                m.destination == Zone::Battlefield
                    && (m.before.card.aura().is_some() && m.attachment.is_none()
                        || matches!(m.before.card.entry_tapped(), EntryTapped::PayLife(_))
                            && m.life_paid.is_none())
            })
            .map(|(i, _)| i)
            .collect();
        choices.sort_by_key(|&i| (moves[i].controller != self.state.active, i));
        // Counter replacement choices are APNAP independently of the move order.
        // Preserve moves and existing Aura/life indices for the simultaneous commit.
        let mut counter_order = moves
            .iter()
            .enumerate()
            .filter(|(_, m)| m.destination == Zone::Battlefield && m.counters.total() > 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        counter_order.sort_by_key(|&i| (moves[i].controller != self.state.active, i));
        let batch = EntryBatch {
            moves,
            choices,
            next: 0,
            counter_next: 0,
            counter_order,
            resume: None,
            shuffle_after,
            replacement_resume: None,
        };
        let counter_replacements = batch.moves.iter().any(|m| {
            m.destination == Zone::Battlefield
                && m.counters.total() > 0
                && (!m.before.card.replacement_effects().is_empty()
                    || self.state.static_sources.iter().any(|&id| {
                        self.state.objects[id as usize]
                            .card
                            .replacement_effects()
                            .iter()
                            .any(|e| matches!(e, crate::cards::ReplacementSpec::Counters { .. }))
                    }))
        });
        if batch.choices.is_empty() && !counter_replacements {
            self.commit_entries(batch);
        } else {
            self.state.entry_batch = Some(Box::new(batch));
        }
    }
    pub(super) fn has_entry_counter_replacements(&self, card: Card) -> bool {
        (card.enters_counters().total() > 0 || !card.entry_counter_rules().is_empty())
            && (card
                .replacement_effects()
                .iter()
                .any(|e| matches!(e, crate::cards::ReplacementSpec::Counters { .. }))
                || self.state.static_sources.iter().any(|&id| {
                    self.state.objects[id as usize]
                        .card
                        .replacement_effects()
                        .iter()
                        .any(|e| matches!(e, crate::cards::ReplacementSpec::Counters { .. }))
                }))
    }
    pub(super) fn pre_entry_counters(&self, before: &ObjectView, destination: Zone) -> Counters {
        self.pre_entry_counters_with_payment(before, destination, None, before.controller)
    }
    fn pre_entry_counters_with_payment(
        &self,
        before: &ObjectView,
        destination: Zone,
        payment: Option<Mana>,
        controller: Player,
    ) -> Counters {
        if destination != Zone::Battlefield {
            return Counters::default();
        }
        let mut counters = before.card.enters_counters();
        for rule in before.card.entry_counter_rules() {
            if self.step_guard(rule.guard, controller, before.reference, before.tapped) {
                let amount = match rule.amount {
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
    fn commit_entries(&mut self, batch: EntryBatch) {
        // Apply quantities captured before this event's replacement choices.
        // No incoming permanent or same-event life payment changes those values.
        let moves: Vec<_> = batch
            .moves
            .into_iter()
            .filter(|m| !m.skipped)
            .map(|m| {
                // Impossible Aura entry can redirect an already staged move.
                let counters = if m.destination == Zone::Battlefield {
                    m.counters
                } else {
                    Counters::default()
                };
                (m, counters)
            })
            .collect();
        let mut events = Vec::new();
        let mut attachments = Vec::new();
        let mut role_results = Vec::new();
        for (movement, counters) in moves {
            let mut event = self.transition_object_under_control(
                movement.before,
                movement.destination,
                movement.tapped,
                counters,
                Some(movement.controller),
            );
            event.offspring_paid = movement.offspring_paid;
            event.kicked = movement.kicked;
            event.bargained = movement.bargained;
            event.x = movement.x;
            event.evidence_collected = movement.evidence_collected;
            if event.moved {
                if event.destination == Zone::Battlefield
                    && let Some(token) = movement.role_after
                {
                    role_results.push((movement.controller, token, event.after));
                }
                if event.destination == Zone::Battlefield
                    && let Some(defender) = movement.attacking
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
                self.register_warp_exile(&event, controller);
            }
            events.push(event);
        }
        // Continuous attachment effects already exist when entry events inspect the world.
        for (source, target) in attachments {
            self.state.objects[source.card as usize].attached_to = Some(target);
            if self.state.objects[source.card as usize]
                .card
                .role()
                .is_some()
            {
                let timestamp = self.next_role_timestamp();
                self.record_role(source, target, timestamp);
            }
        }
        self.finish_zone_events(&events, &[]);
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
            self.state
                .rng
                .shuffle(&mut self.state.players[player.index()].library);
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
            while let Some((index, owner)) = self.state.entry_batch.as_ref().and_then(|batch| {
                batch
                    .choices
                    .get(batch.next)
                    .map(|&index| (index, batch.moves[index].controller))
            }) {
                let movement = &self.state.entry_batch.as_ref().unwrap().moves[index];
                if matches!(movement.before.card.entry_tapped(), EntryTapped::PayLife(_)) {
                    if self.can_pay_entry_life() {
                        self.state.pending = Pending::EntryLife { player: owner };
                        return true;
                    }
                    // An optional positive payment is impossible below the cost.
                    self.choose_entry_life(false);
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
            while self.state.entry_batch.as_ref().unwrap().counter_next
                < self.state.entry_batch.as_ref().unwrap().counter_order.len()
            {
                let batch = self.state.entry_batch.as_mut().unwrap();
                let index = batch.counter_order[batch.counter_next];
                batch.counter_next += 1;
                let m = &batch.moves[index];
                if m.destination != Zone::Battlefield || m.skipped || m.counters.total() == 0 {
                    continue;
                }
                let p = replacements::CounterPlacement {
                    object: m.before.reference,
                    card: m.before.card,
                    owner: m.before.owner,
                    controller: m.controller,
                    subtypes: GrantedSubtypes::default(),
                    kind: m.before.card.kind_mask(),
                    amount: m.counters,
                    effect: true,
                    entry_index: Some(index),
                    private: !visible(m, m.before.owner.other(), &self.state.known_hand),
                };
                self.stage_counter_placements(vec![p], replacements::ResumeMode::Entry);
                if self.advance_replacements(false) {
                    return true;
                }
            }
            let mut batch = *self.state.entry_batch.take().unwrap();
            self.state.pending = batch.resume.clone().unwrap();
            let replacement_resume = batch.replacement_resume.take();
            self.commit_entries(batch);
            if let Some(outer) = replacement_resume {
                self.state.replacements = Some(outer);
                if self.advance_replacements(true) {
                    return true;
                }
            } else if self.state.resolving.is_some() {
                self.run_program();
            }
            if self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| a.additional_paid)
            {
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
    pub(super) fn can_pay_entry_life(&self) -> bool {
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[batch.choices[batch.next]];
        let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() else {
            unreachable!()
        };
        self.state.players[movement.controller.index()].life >= i128::from(cost)
    }
    pub(super) fn choose_entry_life(&mut self, pay: bool) {
        assert!(!pay || self.can_pay_entry_life());
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
            self.lose_life(player, i128::from(cost));
        }
    }
    pub(super) fn entry_life_view(&self, viewer: Player) -> Option<EntryLifeView> {
        if !matches!(self.state.pending, Pending::EntryLife { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = batch.moves.get(*batch.choices.get(batch.next)?)?;
        if !visible(movement, viewer, &self.state.known_hand) {
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
            selected: batch.selected_life(viewer, &self.state.known_hand),
            selected_attachments: batch.selected_attachments(viewer, &self.state.known_hand),
        })
    }
    pub(super) fn aura_entry_view(&self, viewer: Player) -> Option<AuraEntryView> {
        if !matches!(self.state.pending, Pending::AuraEntry { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = batch.moves.get(*batch.choices.get(batch.next)?)?;
        if !visible(movement, viewer, &self.state.known_hand) {
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
            selected: batch.selected_attachments(viewer, &self.state.known_hand),
            selected_life: batch.selected_life(viewer, &self.state.known_hand),
        })
    }
}
