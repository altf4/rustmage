//! Cold named counters live outside the hot, trivially cloned object array.
//! Rows are canonical by card slot and qualified by its exact incarnation.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct NamedCounterRow {
    object: ObjectRef,
    amounts: NamedCounters,
}
pub(super) type Rows = Option<SharedVec<NamedCounterRow>>;

pub(super) fn amounts(rows: &Rows, object: ObjectRef, hot: InitialCounters) -> CounterAmounts<'_> {
    CounterAmounts {
        hot,
        named: rows.as_ref().and_then(|rows| {
            rows.binary_search_by_key(&object.card, |r| r.object.card)
                .ok()
                .and_then(|i| (rows[i].object == object).then_some(&rows[i].amounts))
        }),
    }
}
pub(super) fn set(rows: &mut Rows, object: ObjectRef, named: Option<Box<NamedCounters>>) {
    if rows.is_none() && named.is_none() {
        return;
    }
    let values = rows.get_or_insert_with(SharedVec::default);
    match values.binary_search_by_key(&object.card, |r| r.object.card) {
        Ok(i) => {
            if let Some(amounts) = named {
                values[i] = NamedCounterRow {
                    object,
                    amounts: *amounts,
                };
            } else {
                values.remove(i);
            }
        }
        Err(i) => {
            if let Some(amounts) = named {
                values.insert(
                    i,
                    NamedCounterRow {
                        object,
                        amounts: *amounts,
                    },
                );
            }
        }
    }
    if values.is_empty() {
        *rows = None;
    }
}
pub(super) fn same_storage(left: &Rows, right: &Rows) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => a.ptr_eq(b),
        _ => false,
    }
}
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<NamedCounterRow>>()
            + rows.len() * std::mem::size_of::<NamedCounterRow>()
            + rows
                .iter()
                .map(|r| r.amounts.0.len() * std::mem::size_of::<(CounterKind, u64)>())
                .sum::<usize>()
    })
}
impl Game {
    pub(super) fn counter_amounts(&self, id: CardId) -> CounterAmounts<'_> {
        amounts(
            &self.state.named_counters,
            self.object_ref(id),
            self.state.objects[id as usize].counters,
        )
    }
    /// Current counters of the exact live incarnation, in any zone.
    pub(super) fn counter_count(&self, id: CardId, kind: CounterKind) -> u64 {
        self.counter_amounts(id).get(kind)
    }
    pub(super) fn set_counters(&mut self, id: CardId, counters: Counters) {
        let before = self.counter_amounts(id).keyword_amounts();
        let after = CounterAmounts::from(&counters).keyword_amounts();
        let hot = counters.hot();
        if self.state.objects[id as usize].counters != hot {
            self.state.objects[id as usize].counters = hot;
        }
        let object = self.object_ref(id);
        set(&mut self.state.named_counters, object, counters.named);
        if before != after {
            keyword_counters::sync_effects(
                self.state.roles.get_or_insert_with(Default::default),
                object,
                before,
                after,
                None,
            );
        }
    }
    pub(super) fn add_counter(&mut self, id: CardId, kind: CounterKind, amount: u64) {
        if amount == 0 {
            return;
        }
        let mut counters = self.counter_amounts(id).to_owned();
        counters.add(kind, amount);
        self.set_counters(id, counters);
    }
    /// Move only counters on the unchanged live source, never its departure LKI.
    pub(super) fn move_counters_from_source(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        amount: AmountValue,
    ) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let Some(target) = self
            .keyword_subjects(item, Subject::Targets)
            .first()
            .copied()
        else {
            return;
        };
        if target == item.card {
            return;
        }
        let amount = self
            .item_amount(amount, item)
            .min(u128::from(self.counter_count(item.card, kind))) as u64;
        if amount == 0 {
            return;
        }
        self.pay_counters(item.card, &Counters::of(kind, amount));
        self.effect_counters(
            vec![(self.object_ref(target), Counters::of(kind, amount))],
            replacements::ResumeMode::Program,
        );
    }
    pub(super) fn pay_counters(&mut self, id: CardId, payment: &Counters) {
        if payment.is_empty() {
            return;
        }
        let mut counters = self.counter_amounts(id).to_owned();
        counters.pay(payment);
        self.set_counters(id, counters);
    }
}
