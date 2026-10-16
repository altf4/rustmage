//! Explicit scalar declarations and complete interval domains. Work is bounded by
//! immutable threshold breakpoints and at most 33 funding queries per segment, never X.
use super::*;
use crate::cards::CastFilter;
impl Game {
    pub(super) fn scalar_x_total(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
    ) -> Option<ManaCost> {
        let card = self.state.objects[id as usize].card;
        let mana_value = card.fixed_mana_value() + u64::from(card.x_symbols()) * u64::from(x);
        let raw = u64::from(base.generic) + u64::from(card.x_symbols()) * u64::from(x);
        let discount = self.spell_reduction_at_mana_value(actor, id, mana_value);
        Some(ManaCost {
            colored: base.colored,
            generic: u32::try_from(raw.saturating_sub(discount)).ok()?,
        })
    }
    pub(super) fn scalar_x_intervals(&self, actor: Player, id: CardId) -> Vec<XInterval> {
        let card = self.state.objects[id as usize].card;
        let symbols = u64::from(card.x_symbols());
        if symbols == 0 {
            return Vec::new();
        }
        let end = u64::from(u32::MAX) + 1;
        let mut cuts = vec![0, end];
        fn thresholds(filter: CastFilter, base: u64, symbols: u64, end: u64, cuts: &mut Vec<u64>) {
            match filter {
                CastFilter::ManaValueMinimum { filter, minimum } => {
                    cuts.push(minimum.saturating_sub(base).div_ceil(symbols).min(end));
                    thresholds(*filter, base, symbols, end, cuts);
                }
                CastFilter::ManaValueMaximum { filter, maximum } => {
                    cuts.push(
                        maximum
                            .saturating_add(1)
                            .saturating_sub(base)
                            .div_ceil(symbols)
                            .min(end),
                    );
                    thresholds(*filter, base, symbols, end, cuts);
                }
                CastFilter::AnyOf(filters) => {
                    for f in filters {
                        thresholds(*f, base, symbols, end, cuts);
                    }
                }
                CastFilter::Targeted { filter, .. } => {
                    thresholds(*filter, base, symbols, end, cuts)
                }
                _ => {}
            }
        }
        for rule in card.cost_reductions() {
            thresholds(
                rule.filter,
                card.fixed_mana_value(),
                symbols,
                end,
                &mut cuts,
            );
        }
        for &source in &self.state.static_sources {
            let o = &self.state.objects[source as usize];
            if o.owner == actor {
                for rule in o.card.cost_reductions() {
                    thresholds(
                        rule.filter,
                        card.fixed_mana_value(),
                        symbols,
                        end,
                        &mut cuts,
                    );
                }
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        let (colored, generic) = card.cost();
        let base = ManaCost { colored, generic };
        let fundable = |x| {
            self.scalar_x_total(actor, id, base, x)
                .is_some_and(|cost| self.spell_can_fund_total(actor, id, cost, None))
        };
        let mut intervals: Vec<XInterval> = Vec::new();
        for ends in cuts.windows(2) {
            let lo = ends[0] as u32;
            let hi = (ends[1] - 1) as u32;
            if !fundable(lo) {
                continue;
            }
            let (mut left, mut right) = (u64::from(lo), u64::from(hi));
            while left < right {
                let mid = left + (right - left).div_ceil(2);
                if fundable(mid as u32) {
                    left = mid;
                } else {
                    right = mid - 1;
                }
            }
            let range = XInterval {
                minimum: lo,
                maximum: left as u32,
            };
            if let Some(last) = intervals.last_mut()
                && u64::from(last.maximum) + 1 == u64::from(range.minimum)
            {
                last.maximum = range.maximum;
            } else {
                intervals.push(range);
            }
        }
        intervals
    }
    pub fn x_choice(&self) -> Option<XChoiceView> {
        if !matches!(self.state.pending, Pending::X { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        Some(XChoiceView {
            card: a.item.object,
            intervals: self.scalar_x_intervals(a.item.controller, a.item.card),
        })
    }
    pub(super) fn item_amount(&self, value: AmountValue, item: &SpellView) -> u128 {
        match value {
            AmountValue::ChosenX => u128::from(item.x.unwrap_or(0)),
            AmountValue::Multiple { value, factor } => self
                .item_amount(*value, item)
                .checked_mul(u128::from(factor))
                .capacity("effect amount representation exhausted"),
            other => self.amount_value(other, item.object, item.controller),
        }
    }
}
