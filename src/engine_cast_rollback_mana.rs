//! Selective CR733.1 reversal of completed, frozen mana events.
//! These cold actual-event reconstructions never participate in funding search.
use super::*;
use cast_rollback::CastRollback;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ManaEventKind {
    Basic {
        source: ObjectRef,
    },
    Granted {
        source: ObjectRef,
        sacrifice: bool,
    },
    Activated {
        item: Box<SpellView>,
        payment: Mana,
        counters: Counters,
        additional: Option<(AdditionalCost, ObjectRef)>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManaEvent {
    pub id: u32,
    pub actor: Player,
    pub kind: ManaEventKind,
    pub production: Mana,
    pub quality: Option<mana_lots::Quality>,
    pub spent_lots: Vec<(mana_lots::Quality, Mana)>,
    pub entries: Vec<entries::EntryBatch>,
    pub entry_life: [u128; 2],
    pub moves: Vec<(ObjectRef, Zone, bool)>,
    pub allocations: Vec<(CardId, u64)>,
    pub triggers: Vec<SpellView>,
    pub irreversible: bool,
}
impl ManaEvent {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .entries
                .iter()
                .map(entries::EntryBatch::bytes)
                .sum::<usize>()
            + self
                .triggers
                .iter()
                .map(|item| std::mem::size_of::<SpellView>() + item.owned_bytes())
                .sum::<usize>()
            + self.spent_lots.len() * std::mem::size_of::<(mana_lots::Quality, Mana)>()
            + self.moves.len() * std::mem::size_of::<(ObjectRef, Zone, bool)>()
            + self.allocations.len() * std::mem::size_of::<(CardId, u64)>()
            + match &self.kind {
                ManaEventKind::Activated { item, counters, .. } => {
                    std::mem::size_of::<SpellView>() + item.owned_bytes() + counters.owned_bytes()
                }
                _ => 0,
            }
    }
    fn source(&self) -> ObjectRef {
        match &self.kind {
            ManaEventKind::Basic { source } | ManaEventKind::Granted { source, .. } => *source,
            ManaEventKind::Activated { item, .. } => item.object,
        }
    }
    fn payment(&self) -> Mana {
        match &self.kind {
            ManaEventKind::Activated { payment, .. } => *payment,
            _ => [0; 6],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveManaReplay {
    entries: VecDeque<entries::EntryBatch>,
    entry_life: [u128; 2],
    moves: VecDeque<(ObjectRef, Zone, bool)>,
    allocations: VecDeque<(CardId, u64)>,
    failed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManaReplay {
    current: State,
    current_bytes: usize,
    index: usize,
    active: Option<Box<ActiveManaReplay>>,
}
impl ManaReplay {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.current_bytes
            + self.active.as_ref().map_or(0, |active| {
                std::mem::size_of::<ActiveManaReplay>()
                    + active
                        .entries
                        .iter()
                        .map(entries::EntryBatch::bytes)
                        .sum::<usize>()
                    + active.moves.len() * std::mem::size_of::<(ObjectRef, Zone, bool)>()
                    + active.allocations.len() * std::mem::size_of::<(CardId, u64)>()
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManaRollbackView {
    pub event: u32,
    pub index: u32,
    pub total: u32,
    pub source: ObjectRef,
    pub definition: Card,
    pub production: Mana,
    pub payment: Mana,
    pub irreversible: bool,
    pub can_keep: bool,
    pub can_reverse: bool,
}
impl Game {
    pub(super) fn start_mana_rollback(
        &mut self,
        mut rollback: Box<CastRollback>,
        current: State,
    ) -> bool {
        if rollback.events.is_empty() {
            return false;
        }
        let checkpoint = Game {
            state: current,
            revision: 0,
        };
        let current_bytes = checkpoint.state_bytes();
        rollback.replay = Some(Box::new(ManaReplay {
            current: checkpoint.state,
            current_bytes,
            index: 0,
            active: None,
        }));
        self.state = rollback.before.clone();
        self.restore_public_hand_knowledge(
            &rollback.replay.as_ref().unwrap().current,
            &rollback.public_hand_cards,
            &rollback.obscured_hand_cards,
        );
        self.restore_public_face_down(
            &rollback.replay.as_ref().unwrap().current,
            &rollback.public_face_down,
            &rollback.obscured_face_down,
        );
        let player = rollback.player;
        self.state.cast_rollback = Some(rollback);
        self.state.pending = Pending::ManaRollback { player };
        true
    }
    pub(super) fn mana_rollback_view(&self, viewer: Player) -> Option<ManaRollbackView> {
        if !matches!(self.state.pending, Pending::ManaRollback { .. }) {
            return None;
        }
        let rollback = self.state.cast_rollback.as_ref()?;
        if !self.shares_information(viewer, rollback.player) {
            return None;
        }
        let replay = rollback.replay.as_ref()?;
        let event = rollback.events.get(replay.index)?;
        let source = event.source();
        let definition = match &event.kind {
            ManaEventKind::Activated { item, .. } => item.source_card,
            _ => self.state.objects[source.card as usize].card,
        };
        Some(ManaRollbackView {
            event: event.id,
            index: replay.index as u32,
            total: rollback.events.len() as u32,
            source,
            definition,
            production: event.production,
            payment: event.payment(),
            irreversible: event.irreversible,
            can_keep: self.mana_rollback_option(true),
            can_reverse: self.mana_rollback_option(false),
        })
    }
    pub(super) fn visit_mana_rollback_actions(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for keep in [false, true] {
            if self.mana_rollback_option(keep) && emit(Action::ChooseManaRollback { keep }) {
                return true;
            }
        }
        false
    }
    fn mana_rollback_option(&self, keep: bool) -> bool {
        let Some(rollback) = self.state.cast_rollback.as_ref() else {
            return false;
        };
        let Some(replay) = rollback.replay.as_ref() else {
            return false;
        };
        let Some(event) = rollback.events.get(replay.index) else {
            return false;
        };
        if !keep && event.irreversible {
            return false;
        }
        let mut branch = Game {
            state: self.state.clone(),
            revision: self.revision,
        };
        (!keep || branch.replay_completed_mana_event(event))
            && branch.mana_rollback_suffix(&rollback.events[replay.index + 1..])
    }
    fn mana_rollback_suffix(&self, events: &[ManaEvent]) -> bool {
        // Every reversible suffix can be declined immediately. Only future
        // irreversible activations constrain earlier retention decisions.
        if !events.iter().any(|e| e.irreversible) {
            return true;
        }
        let Some((event, rest)) = events.split_first() else {
            return true;
        };
        if !event.irreversible && self.mana_rollback_suffix(rest) {
            return true;
        }
        let mut branch = Game {
            state: self.state.clone(),
            revision: self.revision,
        };
        branch.replay_completed_mana_event(event) && branch.mana_rollback_suffix(rest)
    }
    pub(super) fn choose_mana_rollback(&mut self, keep: bool) {
        let (event, last) = {
            let rollback = self.state.cast_rollback.as_ref().unwrap();
            let replay = rollback.replay.as_ref().unwrap();
            (
                rollback.events[replay.index].clone(),
                replay.index + 1 == rollback.events.len(),
            )
        };
        if keep {
            assert!(
                self.replay_completed_mana_event(&event),
                "validated frozen mana retention"
            );
        }
        self.state
            .cast_rollback
            .as_mut()
            .unwrap()
            .replay
            .as_mut()
            .unwrap()
            .index += 1;
        if last {
            let mut rollback = self.state.cast_rollback.take().unwrap();
            let replay = rollback.replay.take().unwrap();
            self.state.pending = rollback.before.pending.clone();
            self.complete_cast_rollback_restore(
                &replay.current,
                rollback.spell,
                &rollback.public_hand_cards,
                &rollback.obscured_hand_cards,
                &rollback.public_face_down,
                &rollback.obscured_face_down,
            );
        }
    }
    fn prepare_frozen_mana_spend(
        &mut self,
        event: &ManaEvent,
        purpose: mana_lots::Purpose,
    ) -> bool {
        let payment = event.payment();
        if payment == [0; 6] {
            return true;
        }
        let pool = self.state.players[event.actor.index()].mana.map(u128::from);
        let lots = self.mana_lots(event.actor);
        let ordinary = mana_lots::ordinary(pool, lots);
        let mut allocations = vec![[0; 6]; lots.len()];
        let mut ordinary_payment = payment;
        for &(quality, amounts) in &event.spent_lots {
            let Some(index) = lots.iter().position(|lot| lot.quality == quality) else {
                return false;
            };
            if !quality.eligible(purpose) {
                return false;
            }
            for color in 0..6 {
                if amounts[color] > ordinary_payment[color]
                    || u128::from(allocations[index][color]) + u128::from(amounts[color])
                        > lots[index].amounts[color]
                {
                    return false;
                }
                allocations[index][color] += amounts[color];
                ordinary_payment[color] -= amounts[color];
            }
        }
        if (0..6).any(|i| u128::from(ordinary_payment[i]) > ordinary[i]) {
            return false;
        }
        if !lots.is_empty() {
            self.state.mana_lots.as_mut().unwrap().replay = Some(allocations);
        }
        true
    }
    fn replay_completed_mana_event(&mut self, event: &ManaEvent) -> bool {
        // Completion checks inspect fixed paid resources, not an ability's
        // present guard or the production its current characteristics would yield.
        let source = event.source();
        let Some(object) = self.state.objects.get(source.card as usize) else {
            return false;
        };
        let purpose = match &event.kind {
            ManaEventKind::Activated { item, .. } => self.purpose_for_item(item),
            _ => mana_lots::Purpose::Other,
        };
        match &event.kind {
            ManaEventKind::Basic { .. } | ManaEventKind::Granted { .. } => {
                if self.object_ref(source.card) != source
                    || object.zone != Zone::Battlefield
                    || object.tapped
                {
                    return false;
                }
            }
            ManaEventKind::Activated {
                item,
                counters,
                additional,
                ..
            } => {
                let StackKind::ActivatedAbility { ability } = item.kind else {
                    return false;
                };
                let definition = item.source_card.activation(ability);
                let cost = definition.cost;
                if self.state.players[event.actor.index()].life < i128::from(cost.life)
                    && cost.life > 0
                {
                    return false;
                }
                let source_cost = cost.tap
                    || cost.sacrifice
                    || cost.discard
                    || cost.exile_self
                    || cost.remove.is_some()
                    || cost.loyalty.is_some();
                if source_cost
                    && (self.object_ref(source.card) != source
                        || object.zone != definition.source_zone()
                        || (cost.tap && object.tapped)
                        || !self.counter_amounts(source.card).contains(counters))
                {
                    return false;
                }
                if let Some((spec, external)) = additional
                    && (spec.kind != EffectChoiceKind::ExileGraveyard
                        || self.object_ref(external.card) != *external
                        || self.state.objects[external.card as usize].zone != Zone::Graveyard)
                {
                    return false;
                }
            }
        }
        if !self.prepare_frozen_mana_spend(event, purpose) {
            return false;
        }
        let trigger_count = self.state.pending_triggers.len();
        let stack_id = self.state.next_stack_id;
        self.state
            .cast_rollback
            .as_mut()
            .unwrap()
            .replay
            .as_mut()
            .unwrap()
            .active = Some(Box::new(ActiveManaReplay {
            entries: event.entries.clone().into(),
            entry_life: event.entry_life,
            moves: event.moves.clone().into(),
            allocations: event.allocations.clone().into(),
            failed: false,
        }));
        match &event.kind {
            ManaEventKind::Basic { .. } => self.tap_objects(&[source.card]),
            ManaEventKind::Granted { sacrifice, .. } => {
                self.tap_objects(&[source.card]);
                if *sacrifice {
                    self.sacrifice_batch(&[(event.actor, source.card)]);
                }
            }
            ManaEventKind::Activated {
                item,
                payment,
                counters,
                additional,
            } => {
                if let Some((_, external)) = additional {
                    self.move_object(external.card, Zone::Exile);
                }
                let mut item = (**item).clone();
                self.pay_activation(&mut item, *payment, counters.clone());
            }
        }
        self.add_quality_mana(event.actor, event.production, event.quality);
        let active = self
            .state
            .cast_rollback
            .as_mut()
            .unwrap()
            .replay
            .as_mut()
            .unwrap()
            .active
            .take()
            .unwrap();
        if active.failed
            || !active.entries.is_empty()
            || !active.moves.is_empty()
            || !active.allocations.is_empty()
            || self.state.entry_batch.is_some()
            || self.state.replacements.is_some()
        {
            return false;
        }
        self.state.pending_triggers.truncate(trigger_count);
        self.state.next_stack_id = stack_id;
        if let ManaEventKind::Activated { item, .. } = &event.kind {
            self.state.next_stack_id = self.state.next_stack_id.max(
                item.id
                    .checked_add(1)
                    .capacity("retained activation IDs exhausted"),
            );
        }
        for trigger in &event.triggers {
            // Retained mana carries its actual observer packet and consumed limit;
            // an undone event carries neither. Do not recompute its past witnesses.
            self.restore_activation_observer_use(trigger);
            self.state.next_stack_id = self.state.next_stack_id.max(
                trigger
                    .id
                    .checked_add(1)
                    .capacity("retained trigger IDs exhausted"),
            );
            self.state.pending_triggers.push(trigger.clone());
        }
        true
    }
    pub(super) fn replay_frozen_mana_entries(&mut self) -> bool {
        let Some(active) = self
            .state
            .cast_rollback
            .as_mut()
            .and_then(|r| r.replay.as_mut())
            .and_then(|r| r.active.as_mut())
        else {
            return false;
        };
        let Some(mut batch) = active.entries.pop_front() else {
            active.failed = true;
            return true;
        };
        batch.resume = None;
        batch.replacement_resume = None;
        let life = std::mem::take(&mut active.entry_life);
        for player in [Player::P0, Player::P1] {
            let amount =
                i128::try_from(life[player.index()]).capacity("retained entry life exhausted");
            if amount > 0 && self.state.players[player.index()].life < amount {
                self.state
                    .cast_rollback
                    .as_mut()
                    .unwrap()
                    .replay
                    .as_mut()
                    .unwrap()
                    .active
                    .as_mut()
                    .unwrap()
                    .failed = true;
                return true;
            }
            self.lose_life(player, amount);
        }
        self.commit_entries(batch);
        true
    }
    pub(super) fn replay_frozen_mana_zone_plan(
        &mut self,
        source: ObjectRef,
        _requested: Zone,
    ) -> Option<zone_replacements::Plan> {
        let active = self
            .state
            .cast_rollback
            .as_mut()?
            .replay
            .as_mut()?
            .active
            .as_mut()?;
        let Some((expected, destination, shuffle)) = active.moves.pop_front() else {
            active.failed = true;
            return None;
        };
        if expected != source {
            active.failed = true;
        }
        Some(zone_replacements::Plan {
            destination,
            shuffle,
            // Exact actual reflexive packets are restored from ManaEvent.triggers;
            // replay must not recompute their old replacement-source witnesses.
            reflexive: None,
        })
    }
    pub(super) fn replay_frozen_mana_copy_slot(&mut self) -> Option<(CardId, u64)> {
        let active = self
            .state
            .cast_rollback
            .as_mut()?
            .replay
            .as_mut()?
            .active
            .as_mut()?;
        let Some((id, generation)) = active.allocations.pop_front() else {
            active.failed = true;
            return None;
        };
        // Preserve actual identities even if an earlier reversed event had created
        // an object. Inert copy slots represent those now unused allocations.
        while self.state.objects.len() < id as usize {
            let mut empty = self.state.objects[0].clone();
            empty.zone = Zone::Outside;
            empty.copy_kind = CopyKind::Card;
            empty.generation = 0;
            empty.attached_to = None;
            empty.counters = InitialCounters::default();
            let slot = self.state.objects.len() as CardId;
            self.state.objects.push(empty);
            self.state.free_tokens.push(slot);
        }
        if let Some(object) = self.state.objects.get(id as usize)
            && object.zone != Zone::Outside
        {
            self.state
                .cast_rollback
                .as_mut()
                .unwrap()
                .replay
                .as_mut()
                .unwrap()
                .active
                .as_mut()
                .unwrap()
                .failed = true;
        }
        self.state.free_tokens.retain(|&slot| slot != id);
        Some((id, generation))
    }
}
