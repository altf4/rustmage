//! Generic reductions and the total cost locked before mana abilities/payment.
use super::*;
impl Game {
    pub(super) fn spell_cost_option_available(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
    ) -> bool {
        let definition = self.effective_spell_card(card);
        let (colored, generic) = definition.cost();
        cost == ManaCost { colored, generic }
            || definition.cost_alternative_guard().is_none_or(|guard| {
                self.step_guard(
                    guard,
                    actor,
                    self.object_ref(card),
                    self.state.objects[card as usize].tapped,
                )
            })
    }
    pub(super) fn kicked_cast_targets_available(&self, actor: Player, card: CardId) -> bool {
        let definition = self.effective_spell_card(card);
        definition.kicked_targets().is_some_and(|plan| {
            self.cast_targets_available(plan, actor, self.object_ref(card))
                && definition.cost_options().any(|cost| {
                    self.spell_can_fund(actor, card, Self::with_kicker_cost(cost, definition, true))
                })
        })
    }
    pub(super) fn announcement_base_cost(a: &Announcement) -> ManaCost {
        let mut base = a.cost.unwrap_or_else(|| {
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Sneak) {
                return a.sneak_base_costs().next().expect("captured Sneak cost");
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Warp) {
                return a.item.source_card.warp_cost().expect("fixed Warp cost");
            }
            if a.item
                .cast
                .is_some_and(|c| matches!(c.method, CastMethod::Plot | CastMethod::Free))
            {
                return ManaCost {
                    colored: [0; 6],
                    generic: 0,
                };
            }
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
        // A selected hybrid equivalent already includes the declared surcharge.
        if a.cost.is_none() {
            base = Self::with_flash_surcharge(
                base,
                a.item.source_card,
                a.item.cast.is_some_and(|c| c.flash_surcharge),
            );
        }
        if a.cost.is_none() {
            base = Self::with_tiered_mana(base, &a.item).expect("validated selected Tiered cost");
        }
        if a.forage
            .as_ref()
            .is_some_and(|f| f.route == Some(ForageRoute::Black))
        {
            base.colored[2] = base.colored[2]
                .checked_add(1)
                .capacity("forage alternative mana cost exhausted");
        }
        if a.item.cast.is_some_and(|c| c.evidence == Some(true)) {
            base.generic = base
                .generic
                .saturating_sub(a.item.source_card.evidence_reduction());
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
    pub(super) fn with_flash_surcharge(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.flash_surcharge().expect("flash surcharge recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("flash surcharge mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("flash surcharge mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn flash_surcharge_cast_fundable(
        &self,
        actor: Player,
        card: CardId,
        flashback: bool,
    ) -> bool {
        let definition = self.state.objects[card as usize].card;
        let available = match definition.effect() {
            Effect::Modal(modes) => modes.iter().any(|mode| {
                self.cast_targets_available(mode.targets, actor, self.object_ref(card))
            }),
            Effect::Spell(plan, _) | Effect::Aura(plan) => {
                self.cast_targets_available(plan, actor, self.object_ref(card))
            }
            _ => true,
        };
        if !available {
            return false;
        }
        let costs = if flashback {
            std::iter::once(definition.flashback_cost().unwrap()).chain([].iter().copied())
        } else {
            definition.cost_options()
        };
        costs.into_iter().any(|cost| {
            let cost = Self::with_flash_surcharge(cost, definition, true);
            if definition.target_cost_reduction().is_some() {
                self.initial_target_cost_fundable(actor, card, cost)
            } else if definition.collect_evidence_cost().is_some() {
                self.evidence_cast_fundable(actor, card, cost, false)
                    || self.evidence_cast_fundable(actor, card, cost, true)
            } else if definition.has_bargain() {
                self.initial_bargain_fundable(actor, card, cost)
            } else {
                self.spell_can_fund(actor, card, cost)
            }
        })
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
        if proposal.item.source_card.kicked_targets().is_some()
            && !self.targets_available(
                self.target_spec(&proposal.item),
                proposal.item.controller,
                proposal.item.target_source(),
            )
        {
            return false;
        }
        proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }
    /// Nonmana kicker shares the base spell's mana and exact sacrifice witness.
    pub(super) fn kicker_additional_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
        let spec = a
            .item
            .source_card
            .kicker_additional_cost()
            .expect("declared nonmana kicker");
        let mut proposal = a.clone();
        if proposal.cost.is_some_and(|old| old != cost) {
            proposal.total_cost = None;
        }
        proposal.cost = Some(cost);
        let goal = self.funding_goal(&proposal);
        let purpose = self.purpose_for_item(&proposal.item);
        if let Some(object) = a.additional {
            return self.additional_candidate(spec, a.item.controller, a.item.object, object)
                && self.can_fund_for(goal, purpose);
        }
        if self.linked_funding_can_probe(a.item.controller)
            && let Some(result) = self.linked_deferred_funding(
                goal.clone(),
                purpose,
                linked_funding::DeferredFundingCost::Additional {
                    source: a.item.object,
                    spec,
                },
            )
        {
            return result;
        }
        (0..self.state.objects.len()).any(|id| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(spec, a.item.controller, a.item.object, object)
                && self.can_fund_for(
                    FundingGoal {
                        reserve: Some((object, spec)),
                        ..goal.clone()
                    },
                    purpose,
                )
        })
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
        if a.item.cast.is_some_and(|c| c.method == CastMethod::Sneak) {
            let mut proposal = a.clone();
            proposal.item.cast.as_mut().unwrap().offspring = Some(pay);
            return proposal
                .cost_options()
                .any(|cost| self.announcement_cost_fundable(&proposal, cost));
        }
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
            self.effective_spell_card(id).fixed_mana_value(),
        ))
        .unwrap_or(u32::MAX)
    }
    pub(super) fn spell_reduction_at_mana_value(
        &self,
        actor: Player,
        id: CardId,
        mana_value: u64,
    ) -> u64 {
        self.spell_reduction_for_definition(actor, id, mana_value, self.effective_spell_card(id))
    }
    pub(super) fn spell_reduction_for_definition(
        &self,
        actor: Player,
        id: CardId,
        mana_value: u64,
        definition: Card,
    ) -> u64 {
        let recipient = &self.state.objects[id as usize];
        let departing = (recipient.zone == Zone::Hand).then_some(id);
        let applies = |rule: &crate::cards::SpellReduction, source: CardId| {
            let object = &self.state.objects[source as usize];
            (match rule.filter {
                crate::cards::CastFilter::CreatureToughnessGreaterThanPower => {
                    let stats = self.stats_without_hand(id, departing);
                    definition.is_creature() && stats.toughness > stats.power
                }
                crate::cards::CastFilter::SourceChosenCreatureType => self
                    .source_chosen_creature_type(self.object_ref(source))
                    .is_some_and(|kind| definition.has_subtype(kind.name())),
                _ => rule.filter.matches_mana_value(definition, mana_value),
            }) && match rule.turn {
                CostTurn::Any => true,
                CostTurn::Own => self.state.active == actor,
                CostTurn::Other => self.state.active != actor,
            } && self.step_guard_without_hand(
                rule.guard,
                actor,
                self.object_ref(source),
                object.tapped,
                departing,
            )
        };
        let amount = |rule: &crate::cards::SpellReduction, source: CardId| match rule.amount {
            crate::cards::SpellReductionAmount::CreaturePowerCount { minimum } => {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == actor
                            && self.is_creature(*id as CardId)
                            && self.stats_without_hand(*id as CardId, departing).power
                                >= i128::from(minimum)
                    })
                    .count() as u64
            }
            crate::cards::SpellReductionAmount::Fixed(n) => u64::from(n),
            crate::cards::SpellReductionAmount::Attackers { factor } => {
                u64::try_from(self.state.attacked_this_turn.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
            crate::cards::SpellReductionAmount::EquippedPower => self.state.objects
                [source as usize]
                .attached_to
                .filter(|target| {
                    self.on_battlefield(target.card)
                        && self.object_ref(target.card) == *target
                        && Some(target.card) != departing
                        && self.is_creature(target.card)
                })
                .map_or(0, |target| {
                    u64::try_from(self.stats_without_hand(target.card, departing).power.max(0))
                        .unwrap_or(u64::MAX)
                }),
            crate::cards::SpellReductionAmount::SourcePower => {
                u64::try_from(self.stats_without_hand(source, departing).power.max(0))
                    .unwrap_or(u64::MAX)
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
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power);
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
                    .stat_value_without_hand(value, self.object_ref(source), actor, departing)
                    .max(0);
                u64::try_from(n)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
        };
        // Raw scalar requirements fit u64; discounts above u64::MAX are
        // equivalent and impose no artificial reducer-count limit.
        let mut reduction = 0u64;
        for rule in definition.cost_reductions() {
            if rule.source_only && applies(rule, id) {
                reduction = reduction.saturating_add(amount(rule, id));
            }
        }
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            if object.controller != actor {
                continue;
            }
            for (index, rule) in object.card.cost_reductions().iter().enumerate() {
                if !self.continuous_fragment_available(source, |fragment| matches!(fragment,
                    crate::cards::StaticAbilityFragment::CostReduction(slot, _) if usize::from(*slot) == index)) { continue; }
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
