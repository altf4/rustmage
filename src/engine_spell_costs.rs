//! Generic reductions and the total cost locked before mana abilities/payment.
use super::*;
impl Game {
    pub(super) fn announcement_base_cost(a: &Announcement) -> ManaCost {
        let mut base = a.cost.unwrap_or_else(|| {
            if a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Flashback)
            {
                return a.item.source_card.flashback_cost().unwrap();
            }
            assert!(
                !a.item.source_card.has_hybrid_cost(),
                "hybrid cost not chosen"
            );
            let (colored, generic) = a.item.source_card.cost();
            ManaCost { colored, generic }
        });
        if a.forage
            .as_ref()
            .is_some_and(|f| f.route == Some(ForageRoute::Black))
        {
            base.colored[2] = base.colored[2]
                .checked_add(1)
                .capacity("forage alternative mana cost exhausted");
        }
        if a.item.cast.is_some_and(|c| c.bargain == Some(true)) {
            base.generic = base
                .generic
                .saturating_sub(a.item.source_card.bargain_reduction());
        }
        Self::with_kicker_cost(
            Self::with_offspring_cost(
                base,
                a.item.source_card,
                a.item.cast.is_some_and(|c| c.offspring == Some(true)),
            ),
            a.item.source_card,
            a.item.cast.is_some_and(|c| c.kicker == Some(true)),
        )
    }
    pub(super) fn with_kicker_cost(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.kicker_cost().expect("kicker recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("kicker mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("kicker mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn kicker_choice_fundable(&self, a: &Announcement, pay: bool) -> bool {
        let mut proposal = a.clone();
        proposal.item.cast.as_mut().unwrap().kicker = Some(pay);
        proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }
    pub(super) fn with_offspring_cost(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.offspring_cost().expect("offspring recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("offspring mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("offspring mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn offspring_choice_fundable(&self, a: &Announcement, pay: bool) -> bool {
        if a.item.source_card.target_cost_reduction().is_some() {
            return self.announcement_target_fundable(a, None, None, Some(pay), None, false);
        }
        a.cost_options().any(|cost| {
            self.spell_can_fund(
                a.item.controller,
                a.item.card,
                Self::with_offspring_cost(cost, a.item.source_card, pay),
            )
        })
    }
    pub(super) fn spell_reduction(&self, actor: Player, id: CardId) -> u32 {
        u32::try_from(self.spell_reduction_at_mana_value(
            actor,
            id,
            self.state.objects[id as usize].card.fixed_mana_value(),
        ))
        .unwrap_or(u32::MAX)
    }
    pub(super) fn spell_reduction_at_mana_value(
        &self,
        actor: Player,
        id: CardId,
        mana_value: u64,
    ) -> u64 {
        let recipient = &self.state.objects[id as usize];
        let applies = |rule: &crate::cards::SpellReduction, source: CardId| {
            let object = &self.state.objects[source as usize];
            rule.filter.matches_mana_value(recipient.card, mana_value)
                && match rule.turn {
                    CostTurn::Any => true,
                    CostTurn::Own => self.state.active == actor,
                    CostTurn::Other => self.state.active != actor,
                }
                && self.step_guard(rule.guard, actor, self.object_ref(source), object.tapped)
        };
        let amount = |rule: &crate::cards::SpellReduction, source: CardId| match rule.amount {
            crate::cards::SpellReductionAmount::CreaturePowerCount { minimum } => {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.owner == actor
                            && o.card.is_creature()
                            && self.stats(*id as CardId).power >= i128::from(minimum)
                    })
                    .count() as u64
            }
            crate::cards::SpellReductionAmount::Fixed(n) => u64::from(n),
            crate::cards::SpellReductionAmount::Attackers { factor } => {
                u64::try_from(self.state.attacked_this_turn.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
            crate::cards::SpellReductionAmount::SourcePower => {
                u64::try_from(self.stats(source).power.max(0)).unwrap_or(u64::MAX)
            }
            crate::cards::SpellReductionAmount::CreaturePower { filter, greatest } => {
                let mut powers = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, _)| {
                        self.creature_matches(filter, actor, self.object_ref(source), id as CardId)
                    })
                    .map(|(id, _)| self.stats(id as CardId).power);
                // Sum signed values before clamping: negative-power creatures
                // reduce the total, but cannot create a negative discount.
                let value = if greatest {
                    powers.max().unwrap_or(0)
                } else {
                    powers
                        .try_fold(0i128, i128::checked_add)
                        .capacity("spell power aggregate exhausted")
                };
                u64::try_from(value.max(0)).unwrap_or(u64::MAX)
            }
            crate::cards::SpellReductionAmount::Stat { value, factor } => {
                // Widen before multiplying: raw X plus printed generic may exceed
                // u32 before a legal discount makes its payment representable.
                let n = self
                    .stat_value(value, self.object_ref(source), actor)
                    .max(0);
                u64::try_from(n)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
        };
        // Raw scalar requirements fit u64; discounts above u64::MAX are
        // equivalent and impose no artificial reducer-count limit.
        let mut reduction = 0u64;
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
            generic: if base.generic == 0 {
                0
            } else {
                base.generic.saturating_sub(self.spell_reduction(actor, id))
            },
            ..base
        }
    }
}
