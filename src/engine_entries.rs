//! Owned pre-entry attachment/life choices; no object moves until the batch is ready.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryMove {
    pub before: ObjectView,
    pub destination: Zone,
    pub tapped: bool,
    pub attachment: Option<ObjectRef>,
    pub attacking: Option<CombatDefender>,
    pub skipped: bool,
    pub reveal_hand: bool,
    pub life_paid: Option<bool>,
    pub offspring_paid: bool,
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
    pub resume: Option<Pending>,
    pub shuffle_after: Option<Player>,
}
impl EntryBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.moves.len() * std::mem::size_of::<EntryMove>()
            + self.choices.len() * std::mem::size_of::<usize>()
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
            && self.state.objects[target.card as usize].card.is_creature()
            && match spec.relation {
                0 => self.state.objects[target.card as usize].owner == controller,
                1 => self.state.objects[target.card as usize].owner != controller,
                _ => true,
            }
    }
    pub(super) fn entry_move(&self, id: CardId, destination: Zone, tapped: bool) -> EntryMove {
        EntryMove {
            before: self.object_view(id),
            destination,
            tapped,
            attachment: None,
            attacking: None,
            skipped: false,
            reveal_hand: false,
            life_paid: None,
            offspring_paid: false,
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
            movement.counters = self.pre_entry_counters(&movement.before, movement.destination);
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
        choices.sort_by_key(|&i| (moves[i].before.owner != self.state.active, i));
        let batch = EntryBatch {
            moves,
            choices,
            next: 0,
            resume: None,
            shuffle_after,
        };
        if batch.choices.is_empty() {
            self.commit_entries(batch);
        } else {
            self.state.entry_batch = Some(Box::new(batch));
        }
    }
    pub(super) fn pre_entry_counters(&self, before: &ObjectView, destination: Zone) -> Counters {
        if destination != Zone::Battlefield {
            return Counters::default();
        }
        let mut counters = before.card.enters_counters();
        for rule in before.card.entry_counter_rules() {
            if self.step_guard(rule.guard, before.owner, before.reference, before.tapped) {
                let amount = match rule.amount {
                    EntryCounterAmount::Fixed(n) => u64::from(n),
                    EntryCounterAmount::Stat(value) => u64::try_from(
                        self.stat_value(value, before.reference, before.owner)
                            .max(0),
                    )
                    .capacity("entry counter amount exhausted"),
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
        for (movement, counters) in moves {
            let mut event = self.transition_object_with_counters(
                movement.before,
                movement.destination,
                movement.tapped,
                counters,
            );
            event.offspring_paid = movement.offspring_paid;
            event.kicked = movement.kicked;
            event.bargained = movement.bargained;
            event.x = movement.x;
            event.evidence_collected = movement.evidence_collected;
            if event.moved {
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
            events.push(event);
        }
        // Continuous attachment effects already exist when entry events inspect the world.
        for (source, target) in attachments {
            self.state.objects[source.card as usize].attached_to = Some(target);
        }
        self.finish_zone_events(&events, &[]);
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
                    .map(|&index| (index, batch.moves[index].before.owner))
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
            let batch = *self.state.entry_batch.take().unwrap();
            self.state.pending = batch.resume.clone().unwrap();
            self.commit_entries(batch);
            if self.state.resolving.is_some() {
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
        self.aura_can_enchant(movement.before.card, movement.before.owner, target)
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
        self.state.players[movement.before.owner.index()].life >= i128::from(cost)
    }
    pub(super) fn choose_entry_life(&mut self, pay: bool) {
        assert!(!pay || self.can_pay_entry_life());
        let batch = self.state.entry_batch.as_mut().unwrap();
        let movement = &mut batch.moves[batch.choices[batch.next]];
        let EntryTapped::PayLife(cost) = movement.before.card.entry_tapped() else {
            unreachable!()
        };
        let player = movement.before.owner;
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
