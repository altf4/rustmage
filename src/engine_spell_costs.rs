//! Generic reductions and the total cost locked before mana abilities/payment.
use super::*;
impl Game {
    pub(super) fn announcement_base_cost(a: &Announcement) -> ManaCost {
        a.cost.unwrap_or_else(|| {
            assert!(
                !a.item.source_card.has_hybrid_cost(),
                "hybrid cost not chosen"
            );
            let (colored, generic) = a.item.source_card.cost();
            ManaCost { colored, generic }
        })
    }
    pub(super) fn spell_reduction(&self, actor: Player, id: CardId) -> u32 {
        let recipient = &self.state.objects[id as usize];
        // The canonical equivalent has the greatest generic component. No
        // equivalent can benefit from a generic reduction when this is zero.
        if recipient.card.cost().1 == 0 {
            return 0;
        }
        let applies = |rule: &crate::cards::SpellReduction, source: CardId| {
            let object = &self.state.objects[source as usize];
            rule.filter.matches(recipient.card)
                && match rule.turn {
                    CostTurn::Any => true,
                    CostTurn::Own => self.state.active == actor,
                    CostTurn::Other => self.state.active != actor,
                }
                && self.step_guard(rule.guard, actor, self.object_ref(source), object.tapped)
        };
        let amount = |rule: &crate::cards::SpellReduction, source: CardId| match rule.amount {
            crate::cards::SpellReductionAmount::Fixed(n) => n,
            crate::cards::SpellReductionAmount::Stat { value, factor } => {
                // Clamp before multiplying: reductions above u32::MAX are
                // equivalent for every representable generic requirement.
                let n = self
                    .stat_value(value, self.object_ref(source), actor)
                    .max(0);
                u32::try_from(n)
                    .unwrap_or(u32::MAX)
                    .saturating_mul(u32::from(factor))
            }
        };
        // Requirements are u32; saturation is exact for every representable
        // generic cost and avoids an artificial limit on the number of reducers.
        let mut reduction = 0u32;
        for rule in recipient.card.cost_reductions() {
            if rule.source_only && applies(rule, id) {
                reduction = reduction.saturating_add(amount(rule, id));
            }
        }
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            if object.owner != actor {
                continue;
            }
            for rule in object.card.cost_reductions() {
                if !rule.source_only && applies(rule, source) {
                    reduction = reduction.saturating_add(amount(rule, source));
                }
            }
        }
        reduction
    }
    pub(super) fn reduced_spell_cost(&self, actor: Player, id: CardId, base: ManaCost) -> ManaCost {
        ManaCost {
            generic: base.generic.saturating_sub(self.spell_reduction(actor, id)),
            ..base
        }
    }
}
