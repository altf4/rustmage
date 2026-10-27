//! Prospective event-local replacement ordering; no permanent carries this state.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EntryTapKind {
    Tapped,
    Untapped,
    PayLife(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EntryTapEffect {
    pub source: ObjectRef,
    pub intrinsic: bool,
    pub kind: EntryTapKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryTapEvent {
    pub index: usize,
    pub effects: Vec<EntryTapEffect>,
    pub payment: Option<u32>,
}
impl EntryTapEvent {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.effects.len() * std::mem::size_of::<EntryTapEffect>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct EntryTapView {
    pub object: ObjectRef,
    pub card: Card,
    pub tapped: bool,
    pub effects: Vec<EntryTapEffect>,
}
impl Game {
    pub(super) fn entry_tap_effects(&self, movement: &entries::EntryMove) -> Vec<EntryTapEffect> {
        let kinds = if movement.ghost_vacuum_after.is_some() {
            self.ghost_entry_preview(movement)
                .map_or_else(|| movement.entry_kinds(), |view| view.kinds)
        } else {
            movement.entry_kinds()
        };
        let mut effects = Vec::new();
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            if source.zone == Zone::Battlefield
                && source.controller == movement.controller
                && source.card.controlled_entry_untapped() & kinds != 0
                && self.continuous_fragment_available(id, |fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::ControlledEntryUntapped
                    )
                })
            {
                effects.push(EntryTapEffect {
                    source: self.object_ref(id),
                    intrinsic: false,
                    kind: EntryTapKind::Untapped,
                });
            }
        }
        // Preserve the legacy fast path when no new untapped effect applies.
        if effects.is_empty() {
            return effects;
        }
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            if source.zone == Zone::Battlefield
                && source.controller != movement.controller
                && source.card.opponent_entry_tapped() & kinds != 0
                && self.continuous_fragment_available(id, |fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::OpponentEntryTapped(..)
                    )
                })
            {
                effects.push(EntryTapEffect {
                    source: self.object_ref(id),
                    intrinsic: false,
                    kind: EntryTapKind::Tapped,
                });
            }
        }
        effects
    }
    pub(super) fn advance_entry_tap_effects(&mut self) -> bool {
        loop {
            let batch = self.state.entry_batch.as_ref().unwrap();
            let Some(event) = batch.tap_events.get(batch.tap_next) else {
                return false;
            };
            // Recheck for every event: advancing the previous event can expose
            // another land that must choose its type before its tap replacements.
            if batch.moves[event.index].choice.is_none()
                && batch.moves[event.index].before.card.entry_choice()
                    == Some(EntryChoiceKind::BasicLandType)
            {
                self.state.pending = Pending::EntryValue {
                    player: batch.moves[event.index].controller,
                };
                return true;
            }
            let player = batch.moves[event.index].controller;
            if event.payment.is_some() {
                if self.can_pay_entry_life() {
                    self.state.pending = Pending::EntryLife { player };
                    return true;
                }
                self.finish_entry_tap_payment(false);
                continue;
            }
            if event.effects.is_empty() {
                self.state.entry_batch.as_mut().unwrap().tap_next += 1;
                continue;
            }
            // Identical fixed replacements commute; there is no meaningful order
            // choice. A payment always remains a real separate choice.
            let first = event.effects[0];
            let commute = !matches!(first.kind, EntryTapKind::PayLife(_))
                && event.effects.iter().all(|effect| effect.kind == first.kind);
            if event.effects.len() == 1 || commute {
                self.choose_entry_tap_effect(first.source, first.intrinsic);
                continue;
            }
            self.state.pending = Pending::EntryReplacement { player };
            return true;
        }
    }
    pub(super) fn entry_tap_candidates(&self) -> Vec<EntryTapEffect> {
        let batch = self.state.entry_batch.as_ref().unwrap();
        batch.tap_events[batch.tap_next].effects.clone()
    }
    pub(super) fn choose_entry_tap_effect(&mut self, source: ObjectRef, intrinsic: bool) {
        let batch = self.state.entry_batch.as_mut().unwrap();
        let event = &mut batch.tap_events[batch.tap_next];
        let i = event
            .effects
            .iter()
            .position(|effect| effect.source == source && effect.intrinsic == intrinsic)
            .expect("validated entry replacement");
        // Removal records once-per-event use independently of source incarnation
        // changes. No permanent moves or priority intervenes during this batch.
        let effect = event.effects.remove(i);
        match effect.kind {
            EntryTapKind::Tapped => batch.moves[event.index].tapped = true,
            EntryTapKind::Untapped => batch.moves[event.index].tapped = false,
            EntryTapKind::PayLife(cost) => event.payment = Some(cost),
        }
    }
    pub(super) fn entry_tap_payment(&self) -> Option<(&entries::EntryMove, u32)> {
        let batch = self.state.entry_batch.as_ref()?;
        let event = batch.tap_events.get(batch.tap_next)?;
        Some((&batch.moves[event.index], event.payment?))
    }
    pub(super) fn finish_entry_tap_payment(&mut self, pay: bool) -> bool {
        let Some((movement, cost)) = self.entry_tap_payment() else {
            return false;
        };
        let player = movement.controller;
        let batch = self.state.entry_batch.as_mut().unwrap();
        let event = &mut batch.tap_events[batch.tap_next];
        event.payment = None;
        let movement = &mut batch.moves[event.index];
        movement.life_paid = Some(pay);
        if !pay {
            movement.tapped = true;
        }
        if pay {
            self.record_cast_mana_entry_life(player, cost);
            self.lose_life(player, i128::from(cost));
        }
        true
    }
    pub(super) fn entry_tap_view(&self, viewer: Player) -> Option<EntryTapView> {
        if !matches!(self.state.pending, Pending::EntryReplacement { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let event = batch.tap_events.get(batch.tap_next)?;
        let movement = &batch.moves[event.index];
        if !entries::visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        Some(EntryTapView {
            object: movement.before.reference,
            card: movement.before.card,
            tapped: movement.tapped,
            effects: event.effects.clone(),
        })
    }
}
