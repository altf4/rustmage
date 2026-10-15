//! Owned pre-entry attachment choices; no object moves until the whole batch is ready.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryMove {
    pub before: ObjectView,
    pub destination: Zone,
    pub tapped: bool,
    pub attachment: Option<ObjectRef>,
    pub skipped: bool,
    pub reveal_hand: bool,
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
            skipped: false,
            reveal_hand: false,
        }
    }
    pub(super) fn stage_entries(&mut self, moves: Vec<EntryMove>, shuffle_after: Option<Player>) {
        assert!(
            self.state.entry_batch.is_none(),
            "one entry transaction at a time"
        );
        let mut choices: Vec<_> = moves
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.destination == Zone::Battlefield
                    && m.before.card.aura().is_some()
                    && m.attachment.is_none()
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
    fn commit_entries(&mut self, batch: EntryBatch) {
        let mut events = Vec::new();
        let mut attachments = Vec::new();
        for movement in batch.moves {
            if movement.skipped {
                continue;
            }
            let event =
                self.transition_object(movement.before, movement.destination, movement.tapped);
            if event.moved {
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
    pub(super) fn aura_entry_view(&self, viewer: Player) -> Option<AuraEntryView> {
        let batch = self.state.entry_batch.as_ref()?;
        let movement = batch.moves.get(*batch.choices.get(batch.next)?)?;
        let visible = |m: &EntryMove| {
            m.before.owner == viewer || !matches!(m.before.zone, Zone::Hand | Zone::Library)
        };
        if !visible(movement) {
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
                .moves
                .iter()
                .filter(|m| visible(m))
                .filter_map(|m| {
                    m.attachment.map(|attachment| AuraAttachmentView {
                        aura: m.before.reference,
                        card: m.before.card,
                        attachment,
                    })
                })
                .collect(),
        })
    }
}
