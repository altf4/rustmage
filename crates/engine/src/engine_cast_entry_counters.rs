//! Resolved entry-counter replacements belong to one exact permanent spell.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastEntryCounterView {
    pub spell: ObjectRef,
    pub kind: CounterKind,
    pub amount: u64,
    /// A count evaluated when this spell enters. Fixed receipts use `amount`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_controlled_subtypes: Option<CastEntrySubtypeCount>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastEntrySubtypeCount {
    pub player: Player,
    pub subtypes: &'static [&'static str],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<CastEntryCounterView>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CastEntryCounterView>>()
            + rows.len() * std::mem::size_of::<CastEntryCounterView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn add_cast_entry_counters(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        amount: AmountValue,
    ) {
        let Some(spell) = self.live_cast_spell(item) else {
            return;
        };
        let amount =
            u64::try_from(self.item_amount(amount, item)).capacity("cast entry counters exhausted");
        self.record_cast_entry_counters(spell, kind, amount);
    }

    pub(super) fn add_cast_entry_counters_for_subtypes(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        subtypes: &'static [&'static str],
    ) {
        let Some(spell) = self.live_cast_spell(item) else {
            return;
        };
        self.state
            .cast_entry_counters
            .get_or_insert_default()
            .push(CastEntryCounterView {
                spell,
                kind,
                amount: 0,
                per_controlled_subtypes: Some(CastEntrySubtypeCount {
                    player: item.controller,
                    subtypes,
                }),
            });
    }
    pub(super) fn record_cast_entry_counters(
        &mut self,
        spell: ObjectRef,
        kind: CounterKind,
        amount: u64,
    ) {
        if amount == 0 {
            return;
        }
        let rows = self
            .state
            .cast_entry_counters
            .get_or_insert_with(SharedVec::default);
        if let Some(row) = rows.iter_mut().find(|row| {
            row.spell == spell && row.kind == kind && row.per_controlled_subtypes.is_none()
        }) {
            row.amount = row
                .amount
                .checked_add(amount)
                .capacity("cast entry counters exhausted");
        } else {
            rows.push(CastEntryCounterView {
                spell,
                kind,
                amount,
                per_controlled_subtypes: None,
            });
        }
    }
    pub(super) fn has_cast_entry_counters(&self, spell: ObjectRef) -> bool {
        self.state
            .cast_entry_counters
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.spell == spell))
    }
    pub(super) fn forget_cast_entry_counters(&mut self, spell: ObjectRef) {
        if !self.has_cast_entry_counters(spell) {
            return;
        }
        let rows = self.state.cast_entry_counters.as_mut().unwrap();
        rows.retain(|row| row.spell != spell);
        if rows.is_empty() {
            self.state.cast_entry_counters = None;
        }
    }
    pub(super) fn apply_cast_entry_counters(&self, movement: &mut entries::EntryMove) {
        if movement.destination != Zone::Battlefield || movement.before.zone != Zone::Stack {
            return;
        }
        if let Some(rows) = &self.state.cast_entry_counters {
            for row in rows
                .iter()
                .filter(|row| row.spell == movement.before.reference)
            {
                let amount = row.per_controlled_subtypes.map_or(row.amount, |rule| {
                    self.state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && object.controller == rule.player
                                && rule
                                    .subtypes
                                    .iter()
                                    .any(|subtype| self.has_subtype(*id as CardId, subtype))
                        })
                        .count() as u64
                });
                movement.counters.add(row.kind, amount);
            }
        }
    }
}
