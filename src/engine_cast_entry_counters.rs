//! Resolved entry-counter replacements belong to one exact creature spell.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastEntryCounterView {
    pub spell: ObjectRef,
    pub kind: CounterKind,
    pub amount: u64,
}
pub(super) type Rows = Option<SharedVec<CastEntryCounterView>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CastEntryCounterView>>()
            + rows.len() * std::mem::size_of::<CastEntryCounterView>()
    })
}
impl Game {
    pub(super) fn add_cast_entry_counters(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        amount: AmountValue,
    ) {
        let Some(event) = item.cast_event.as_deref() else {
            return;
        };
        if !self
            .state
            .stack
            .iter()
            .any(|spell| spell.kind == StackKind::Spell && spell.object == event.object)
        {
            return;
        }
        let amount =
            u64::try_from(self.item_amount(amount, item)).capacity("cast entry counters exhausted");
        if amount == 0 {
            return;
        }
        let rows = self
            .state
            .cast_entry_counters
            .get_or_insert_with(SharedVec::default);
        if let Some(row) = rows
            .iter_mut()
            .find(|row| row.spell == event.object && row.kind == kind)
        {
            row.amount = row
                .amount
                .checked_add(amount)
                .capacity("cast entry counters exhausted");
        } else {
            rows.push(CastEntryCounterView {
                spell: event.object,
                kind,
                amount,
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
                movement.counters.add(row.kind, row.amount);
            }
        }
    }
}
