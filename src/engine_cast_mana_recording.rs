//! Cold receipts of completed legal mana abilities inside a casting proposal.
//! Speculative funding never calls these hooks or owns these snapshots.
use super::*;
use cast_rollback_mana::{ManaEvent, ManaEventKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManaCapture {
    before: State,
    before_bytes: usize,
    event: ManaEvent,
    produced: bool,
}
impl ManaCapture {
    pub(super) fn bytes(&self) -> usize {
        self.before_bytes + self.event.bytes() + std::mem::size_of::<Self>()
            - std::mem::size_of::<State>()
            - std::mem::size_of::<ManaEvent>()
    }
}
impl Game {
    fn start_cast_mana_capture(
        &mut self,
        actor: Player,
        kind: ManaEventKind,
        production: Mana,
        produced: bool,
    ) {
        let Some(mut rollback) = self.state.cast_rollback.take() else {
            return;
        };
        if rollback.replay.is_some() || rollback.capture.is_some() {
            self.state.cast_rollback = Some(rollback);
            return;
        }
        // Taking the rollback out first prevents recursively copying its checkpoint
        // and previous receipts into every cold recording snapshot.
        let checkpoint = Game {
            state: self.state.clone(),
            revision: 0,
        };
        let before_bytes = checkpoint.state_bytes();
        let event = ManaEvent {
            id: u32::try_from(rollback.events.len()).capacity("mana receipt sequence exhausted"),
            actor,
            kind,
            production,
            quality: None,
            spent_lots: Vec::new(),
            entries: Vec::new(),
            entry_life: [0; 2],
            moves: Vec::new(),
            allocations: Vec::new(),
            triggers: Vec::new(),
            irreversible: false,
        };
        rollback.capture = Some(Box::new(ManaCapture {
            before: checkpoint.state,
            before_bytes,
            event,
            produced,
        }));
        self.state.cast_rollback = Some(rollback);
    }
    pub(super) fn capture_cast_mana_action(&mut self, actor: Player, action: &Action) {
        if self
            .state
            .cast_rollback
            .as_ref()
            .is_none_or(|r| r.replay.is_some() || r.capture.is_some())
        {
            return;
        }
        match *action {
            Action::TapMana { card } => {
                let mut production = [0; 6];
                production[ability_world::CharacteristicWorld::intrinsic_mana_color(self, card)
                    .unwrap()] = 1;
                self.start_cast_mana_capture(
                    actor,
                    ManaEventKind::Basic {
                        source: self.object_ref(card),
                    },
                    production,
                    true,
                );
            }
            Action::TapGrantedMana {
                card,
                grantor,
                production,
            } => {
                let sacrifice = ability_world::CharacteristicWorld::granted_mana(self, card)
                    .into_iter()
                    .find(|g| {
                        g.source == grantor
                            && g.production.in_world(self, card).visit(|p| p == production)
                    })
                    .expect("legal granted mana receipt")
                    .sacrifice;
                self.start_cast_mana_capture(
                    actor,
                    ManaEventKind::Granted {
                        source: self.object_ref(card),
                        sacrifice,
                    },
                    production,
                    false,
                );
            }
            Action::ActivateMana {
                card, production, ..
            }
            | Action::ActivateBorrowedMana {
                card, production, ..
            } => {
                // pay_activation_definition replaces this provisional kind with
                // the exact ordinary/borrowed definition before its first cost.
                self.start_cast_mana_capture(
                    actor,
                    ManaEventKind::Basic {
                        source: self.object_ref(card),
                    },
                    production,
                    false,
                );
            }
            _ => {}
        }
    }
    pub(super) fn capture_announced_cast_mana(&mut self, a: &Announcement) {
        if self
            .state
            .cast_rollback
            .as_ref()
            .is_none_or(|r| r.replay.is_some() || r.capture.is_some())
        {
            return;
        }
        let Some(production) = a.production else {
            return;
        };
        let additional = a.additional_cost().zip(a.additional);
        self.start_cast_mana_capture(
            a.item.controller,
            ManaEventKind::Activated {
                item: Box::new(a.item.clone()),
                payment: a.payment.unwrap_or([0; 6]),
                counters: a.counter_payment.clone(),
                additional,
            },
            production,
            false,
        );
    }
    pub(super) fn record_cast_mana_activation(
        &mut self,
        item: &SpellView,
        payment: Mana,
        counters: &Counters,
    ) {
        let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        else {
            return;
        };
        if let ManaEventKind::Basic { .. } = capture.event.kind {
            capture.event.kind = ManaEventKind::Activated {
                item: Box::new(item.clone()),
                payment,
                counters: counters.clone(),
                additional: None,
            };
        }
    }
    pub(super) fn record_cast_mana_production(
        &mut self,
        actor: Player,
        production: Mana,
        quality: Option<mana_lots::Quality>,
    ) {
        let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        else {
            return;
        };
        assert_eq!(capture.event.actor, actor);
        assert_eq!(capture.event.production, production);
        capture.event.quality = quality;
        capture.produced = true;
    }
    pub(super) fn record_cast_mana_spent_lots(
        &mut self,
        lots: &[mana_lots::Lot],
        allocations: &[Mana],
    ) {
        let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        else {
            return;
        };
        capture
            .event
            .spent_lots
            .extend(
                lots.iter()
                    .zip(allocations)
                    .filter_map(|(lot, &allocation)| {
                        (allocation != [0; 6]).then_some((lot.quality, allocation))
                    }),
            );
    }
    pub(super) fn record_cast_mana_entries(&mut self, batch: &entries::EntryBatch) {
        if let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        {
            capture.event.entries.push(batch.clone());
        }
    }
    pub(super) fn record_cast_mana_entry_life(&mut self, player: Player, amount: u32) {
        if let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        {
            capture.event.entry_life[player.index()] = capture.event.entry_life[player.index()]
                .checked_add(u128::from(amount))
                .capacity("entry life receipt exhausted");
        }
    }
    pub(super) fn record_cast_mana_zone_plan(
        &mut self,
        before: &ObjectView,
        plan: &zone_replacements::Plan,
    ) {
        if let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        {
            capture
                .event
                .moves
                .push((before.reference, plan.destination, plan.shuffle));
            capture.event.irreversible |= plan.shuffle
                || plan.destination == Zone::Library
                || before.zone == Zone::Library && plan.destination != Zone::Stack;
        }
    }
    pub(super) fn record_cast_mana_allocation(&mut self, id: CardId, generation: u64) {
        if let Some(capture) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.capture.as_mut())
        {
            capture.event.allocations.push((id, generation));
        }
    }
    pub(super) fn finish_cast_mana_action(&mut self) {
        if self.state.entry_batch.is_some() || self.state.replacements.is_some() {
            return;
        }
        let Some(rollback) = self.state.cast_rollback.as_mut() else {
            return;
        };
        if rollback.capture.as_ref().is_none_or(|c| !c.produced) {
            return;
        }
        let mut capture = rollback.capture.take().unwrap();
        capture.event.triggers =
            self.state.pending_triggers[capture.before.pending_triggers.len()..].to_vec();
        rollback.events.push(capture.event);
    }
}
