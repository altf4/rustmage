//! Frozen received damage, turn-retained mana and limited linked-exile approvals.
use super::*;
use crate::cards::{ManaRetention, TriggerCondition};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LimitedExilePlayView {
    pub source: ObjectRef,
    pub ability_origin: u64,
    pub player: Player,
    pub expires_turn: u32,
    pub remaining: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LinkedPlayApproval {
    family: LimitedExilePlayView,
    stack_id: u64,
    independent_free: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LimitedExileState {
    pub grants: Vec<LimitedExilePlayView>,
    approvals: Vec<LinkedPlayApproval>,
}
impl LimitedExileState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.grants.len() * std::mem::size_of::<LimitedExilePlayView>()
            + self.approvals.len() * std::mem::size_of::<LinkedPlayApproval>()
    }
}
pub(super) struct ReceivedDamage {
    source: ObjectView,
    amount: u128,
    combat: bool,
}
impl Game {
    pub(super) fn capture_received_damage(&self, events: &[DamageEvent]) -> Vec<ReceivedDamage> {
        let mut rows: Vec<ReceivedDamage> = Vec::new();
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            let Target::Card(id) = event.target else {
                continue;
            };
            if !self.on_battlefield(id)
                || !self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|a| matches!(a.condition.event(), TriggerCondition::SelfDamaged))
            {
                continue;
            }
            let reference = self.object_ref(id);
            if let Some(row) = rows.iter_mut().find(|r| r.source.reference == reference) {
                row.amount = row
                    .amount
                    .checked_add(event.amount as u128)
                    .capacity("received damage amount exhausted");
                row.combat &= event.combat;
            } else {
                rows.push(ReceivedDamage {
                    source: self.object_view(id),
                    amount: event.amount as u128,
                    combat: event.combat,
                });
            }
        }
        rows
    }
    pub(super) fn queue_received_damage(&mut self, rows: Vec<ReceivedDamage>) {
        for row in rows {
            for (index, ability) in row.source.card.triggers().iter().enumerate() {
                if matches!(ability.condition.event(), TriggerCondition::SelfDamaged)
                    && self.queue_trigger(&row.source, index)
                {
                    // For received damage, player identifies the damaged source's
                    // frozen controller; no player was dealt damage by this event.
                    self.state.pending_triggers.last_mut().unwrap().damage_event =
                        Some(DamageTriggerView {
                            dealer: None,
                            player: row.source.controller,
                            amount: row.amount,
                            combat: row.combat,
                        });
                }
            }
        }
    }
    pub(super) fn add_retained_scaled_mana(
        &mut self,
        item: &SpellView,
        color: u8,
        amount: AmountValue,
        retention: ManaRetention,
    ) {
        let mut mana = [0; 6];
        mana[usize::from(color)] = u32::try_from(self.item_amount(amount, item))
            .ok()
            .capacity("retained mana amount exhausted");
        let quality = mana_lots::Quality {
            restriction: mana_lots::ManaRestrictionView::Unrestricted,
            spell_bonus: mana_lots::ManaSpellBonusView::None,
            retention,
            copy_source: None,
        };
        self.add_quality_mana(item.controller, mana, Some(quality));
    }
    pub(super) fn exile_controller_top_linked(&mut self, item: &SpellView, amount: u16) {
        let ids: Vec<_> = self.state.players[item.controller.index()]
            .library
            .iter()
            .rev()
            .copied()
            .take(usize::from(amount))
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let moved: Vec<_> = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile)
            .map(|e| e.after.card)
            .collect();
        self.remember_linked_exile_for_origin(item.object, item.ability_origin(), &moved);
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn grant_linked_exile_play(&mut self, item: &SpellView) {
        let family = LimitedExilePlayView {
            source: item.object,
            ability_origin: item.ability_origin(),
            player: item.controller,
            expires_turn: self.state.turn,
            remaining: 1,
        };
        let data = self
            .state
            .limited_exile_plays
            .get_or_insert_with(Default::default);
        if let Some(row) = data.grants.iter_mut().find(|row| {
            row.source == family.source
                && row.ability_origin == family.ability_origin
                && row.player == family.player
                && row.expires_turn == family.expires_turn
        }) {
            row.remaining = row
                .remaining
                .checked_add(1)
                .capacity("linked exile play allowance exhausted");
        } else {
            data.grants.push(family);
        }
    }
    fn linked_exile_family(&self, player: Player, card: CardId) -> Option<LimitedExilePlayView> {
        let object = self.state.objects.get(card as usize)?;
        if object.zone != Zone::Exile || object.is_token() {
            return None;
        }
        let reference = self.object_ref(card);
        self.state
            .limited_exile_plays
            .as_ref()?
            .grants
            .iter()
            .copied()
            .find(|row| {
                row.player == player
                    && row.remaining > 0
                    && row.expires_turn >= self.state.turn
                    && self.state.linked_choices.as_ref().is_some_and(|data| {
                        data.links.iter().any(|link| {
                            link.source == row.source
                                && link.ability_origin == row.ability_origin
                                && link.exiled == reference
                        })
                    })
            })
    }
    pub(super) fn linked_exile_play_available(&self, player: Player, card: CardId) -> bool {
        self.linked_exile_family(player, card).is_some()
    }
    pub(super) fn prepare_linked_exile_play(
        &self,
        player: Player,
        card: CardId,
    ) -> Option<LinkedPlayApproval> {
        if self.state.objects[card as usize].zone != Zone::Exile
            || self.can_cast_exiled_independent(player, card)
        {
            return None;
        }
        Some(LinkedPlayApproval {
            family: self.linked_exile_family(player, card)?,
            stack_id: 0,
            independent_free: self.can_cast_exiled_free(player, card),
        })
    }
    pub(super) fn remember_linked_play_approval(
        &mut self,
        id: u64,
        approval: Option<LinkedPlayApproval>,
    ) {
        if let Some(mut approval) = approval {
            approval.stack_id = id;
            self.state
                .limited_exile_plays
                .as_mut()
                .unwrap()
                .approvals
                .push(approval);
        }
    }
    fn consume_linked_family(&mut self, family: LimitedExilePlayView) {
        let data = self
            .state
            .limited_exile_plays
            .as_mut()
            .expect("limited permission approval");
        let row = data
            .grants
            .iter_mut()
            .find(|row| {
                row.source == family.source
                    && row.ability_origin == family.ability_origin
                    && row.player == family.player
                    && row.expires_turn == family.expires_turn
            })
            .expect("owned limited permission family");
        row.remaining = row
            .remaining
            .checked_sub(1)
            .capacity("linked exile play allowance exhausted");
    }
    pub(super) fn commit_linked_exile_cast(&mut self, stack_id: u64, free: bool) {
        let Some(data) = self.state.limited_exile_plays.as_mut() else {
            return;
        };
        let Some(index) = data.approvals.iter().position(|a| a.stack_id == stack_id) else {
            return;
        };
        let approval = data.approvals.remove(index);
        if !free || !approval.independent_free {
            self.consume_linked_family(approval.family);
        }
    }
    pub(super) fn commit_linked_exile_land(&mut self, player: Player, card: CardId) {
        if self.state.objects[card as usize].zone != Zone::Exile
            || self.can_play_exiled_independent(player, card)
        {
            return;
        }
        if let Some(family) = self.linked_exile_family(player, card) {
            self.consume_linked_family(family);
        }
    }
}
