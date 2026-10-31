//! Generic reductions and the total cost locked before mana abilities/payment.
use super::*;
impl Game {
    /// Compute the wide pre-reduction generic increase from active sources.
    pub(super) fn opponent_noncreature_spell_tax(&self, actor: Player, definition: Card) -> u64 {
        if definition.kind_mask() & 1 != 0 {
            return 0;
        }
        self.state.static_sources.iter().copied().filter(|&source| {
            self.on_battlefield(source) && self.state.objects[source as usize].controller != actor
        }).map(|source| {
            self.state.objects[source as usize].card.static_ability_groups().iter()
                .flat_map(|group| group.fragments.iter())
                .filter_map(|fragment| {
                    let crate::cards::StaticAbilityFragment::OpponentNoncreatureSpellTax(amount) = *fragment else { return None; };
                    self.continuous_fragment_available(source, |f| matches!(f,
                        crate::cards::StaticAbilityFragment::OpponentNoncreatureSpellTax(n) if *n == amount))
                        .then_some(u64::from(amount))
                }).fold(0u64, u64::saturating_add)
        }).fold(0u64, u64::saturating_add)
    }
    /// Keep increases and all reductions wide until the final payable total.
    #[allow(clippy::too_many_arguments)] // Keep the distinct cost components explicit.
    pub(super) fn spell_total_with_reductions(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        definition: Card,
        mana_value: u64,
        extra_generic: u64,
        discount: ManaCost,
        extra_reduction: u64,
    ) -> Option<ManaCost> {
        let raw = u64::from(base.generic)
            .checked_add(extra_generic)?
            .checked_add(self.opponent_noncreature_spell_tax(actor, definition))?;
        let colored = std::array::from_fn(|i| base.colored[i].saturating_sub(discount.colored[i]));
        let excess = (0..6)
            .map(|i| u64::from(discount.colored[i].saturating_sub(base.colored[i])))
            .sum::<u64>();
        let reductions = self
            .spell_reduction_for_definition(actor, id, mana_value, definition)
            .saturating_add(u64::from(discount.generic))
            .saturating_add(extra_reduction)
            .saturating_add(excess);
        Some(ManaCost {
            colored,
            generic: u32::try_from(raw.saturating_sub(reductions)).ok()?,
        })
    }
    pub(super) fn spell_total_for_definition(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        definition: Card,
    ) -> Option<ManaCost> {
        self.spell_total_with_reductions(
            actor,
            id,
            base,
            definition,
            definition.fixed_mana_value(),
            0,
            ManaCost {
                colored: [0; 6],
                generic: 0,
            },
            0,
        )
    }
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
    pub(super) fn announcement_optional_reduction(a: &Announcement) -> u64 {
        let evidence = a.item.cast.is_some_and(|c| c.evidence == Some(true));
        let bargain = a.item.cast.is_some_and(|c| c.bargain == Some(true));
        u64::from(if evidence {
            a.item
                .source
                .expect("cost announcement has object source")
                .card
                .evidence_reduction()
        } else {
            0
        }) + u64::from(if bargain {
            a.item
                .source
                .expect("cost announcement has object source")
                .card
                .bargain_reduction()
        } else {
            0
        })
    }
    pub(super) fn announcement_base_cost(a: &Announcement) -> ManaCost {
        let mut base = a.cost.unwrap_or_else(|| {
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Miracle) {
                return a.miracle_cost.expect("miracle cost");
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Sneak) {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .sneak_cost()
                    .expect("Sneak cost");
            }
            if a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::Harmonize)
            {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .harmonize_cost()
                    .expect("Harmonize cost");
            }
            if a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::Disguise)
            {
                return ManaCost {
                    colored: [0; 6],
                    generic: 3,
                };
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Warp) {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .warp_cost()
                    .expect("fixed Warp cost");
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Airbend) {
                return ManaCost {
                    colored: [0; 6],
                    generic: 2,
                };
            }
            if a.item.cast.is_some_and(|c| {
                matches!(
                    c.method,
                    CastMethod::ExileLife | CastMethod::Plot | CastMethod::Free
                )
            }) {
                return ManaCost {
                    colored: [0; 6],
                    generic: 0,
                };
            }
            if a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Flashback)
            {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .flashback_cost()
                    .unwrap();
            }
            assert!(
                !a.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .has_hybrid_cost(),
                "hybrid cost not chosen"
            );
            let (colored, generic) = a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .cost();
            ManaCost { colored, generic }
        });
        // A selected hybrid equivalent already includes the declared surcharge.
        if a.cost.is_none() {
            base = Self::with_flash_surcharge(
                base,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
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
        Self::with_kicker_cost(
            Self::with_offspring_cost(
                base,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                a.item.cast.is_some_and(|c| c.offspring == Some(true)),
            ),
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
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
            definition.flashback_cost_options()
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
        if proposal
            .item
            .source
            .expect("cost announcement has object source")
            .card
            .kicked_targets()
            .is_some()
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
        if a.item
            .source
            .expect("cost announcement has object source")
            .card
            .target_cost_reduction()
            .is_some()
        {
            return self.announcement_target_fundable(a, None, None, Some(pay), None, false);
        }
        a.cost_options().any(|cost| {
            self.spell_can_fund(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                Self::with_offspring_cost(
                    cost,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                    pay,
                ),
            )
        })
    }
    /// Announced spells already occupy the stack; retain their actual cast origin.
    /// During nested mana abilities the enclosing announcement remains authoritative.
    fn spell_origin_is_nonhand(&self, actor: Player, id: CardId) -> bool {
        let packet = self
            .state
            .announcing
            .iter()
            .map(|a| &a.item)
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter().map(|f| &f.announcement.item)),
            )
            .chain(self.state.stack.iter())
            .find(|s| s.kind == StackKind::Spell && s.source.is_some_and(|source| source.id == id));
        let origin = packet
            .and_then(|s| s.cast.map(|c| c.origin))
            .unwrap_or(self.state.objects[id as usize].zone);
        origin != Zone::Hand || self.state.objects[id as usize].owner != actor
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
            rule.filter.matches_mana_value(definition, mana_value)
                && match rule.turn {
                    CostTurn::Any => true,
                    CostTurn::NonHand => self.spell_origin_is_nonhand(actor, id),
                    CostTurn::Own => self.state.active == actor,
                    CostTurn::Other => self.state.active != actor,
                }
                && self.step_guard_without_hand(
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
            crate::cards::SpellReductionAmount::GreatestCreatureToughness => {
                let maximum = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.controller == actor
                            && self.is_creature(*id as CardId)
                    })
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).toughness)
                    .max()
                    .unwrap_or(0)
                    .max(0);
                u64::try_from(maximum).unwrap_or(u64::MAX)
            }
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
        let mut reduction = self.turn_artifact_spell_reduction(actor, definition);
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
    pub(super) fn reduced_spell_cost(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> Option<ManaCost> {
        self.spell_total_for_definition(actor, id, base, self.effective_spell_card(id))
    }
}
