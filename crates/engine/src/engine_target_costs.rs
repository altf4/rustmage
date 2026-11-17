//! Target-aware cost proposals. Queries observe current facts; virtual funding
//! never changes which targets qualify. Only the final total enters mutable state.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct Completion<'a> {
    actor: Player,
    source: ObjectRef,
    definition: Card,
    plan: TargetPlan,
    chosen: &'a [BoundTarget],
    group: u16,
    extra: Option<Target>,
    finished: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn visit_opposing_target_life_costs(
        &self,
        actor: Player,
        mut emit: impl FnMut(ObjectRef, u32) -> bool,
    ) -> bool {
        use crate::cards::StaticAbilityFragment::OpponentSpellsTargetingSourceLifeCost;
        let mut query = None;
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone != Zone::Battlefield || object.controller == actor {
                continue;
            }
            let reference = self.object_ref(id);
            for (index, group) in object.card.static_ability_groups().iter().enumerate() {
                for fragment in group.fragments {
                    if let OpponentSpellsTargetingSourceLifeCost(amount) = *fragment
                        && amount != 0
                        && query
                            .get_or_insert_with(|| self.characteristic_query())
                            .continuous_group_available(reference, index, group)
                        && emit(reference, amount)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn cast_requires_target_payment(&self, actor: Player, card: Card) -> bool {
        if card.additional_target_mana().is_some()
            || card.additional_cost().is_some_and(|cost| {
                cost.kind == EffectChoiceKind::CollectEvidenceForTargetManaValue
            })
        {
            return true;
        }
        let has_target = |plan: TargetPlan| match plan {
            TargetPlan::Single(spec) => spec.rule != TargetRule::None,
            TargetPlan::Groups(groups) => groups.iter().any(|spec| spec.rule != TargetRule::None),
        };
        let can_target = match card.effect() {
            Effect::Spell(plan, _) | Effect::Aura(plan) => has_target(plan),
            Effect::Modal(modes) => modes.iter().any(|mode| has_target(mode.targets)),
            _ => false,
        } || card.kicked_targets().is_some_and(has_target)
            || card.bargain_targets().is_some_and(has_target)
            || card
                .gift()
                .is_some_and(|gift| has_target(gift.promised_targets))
            || card
                .teamwork()
                .and_then(|teamwork| teamwork.paid_targets)
                .is_some_and(has_target);
        can_target && self.visit_opposing_target_life_costs(actor, |_, _| true)
    }

    pub(super) fn target_evidence_threshold(&self, a: &Announcement) -> Option<u64> {
        let additional = a.printed_additional_cost()?;
        if additional.kind != EffectChoiceKind::CollectEvidenceForTargetManaValue {
            return None;
        }
        // Independent target occurrences can bind the same permanent (CR 115.3).
        // Each chosen occurrence contributes its current value before payment.
        Some(
            a.item
                .targets
                .iter()
                .filter_map(|bound| bound.object)
                .map(|object| self.object_mana_value(object.card))
                .try_fold(0u64, u64::checked_add)
                .capacity("target evidence threshold exhausted"),
        )
    }

    fn applies_target_life_cost(targets: &[BoundTarget], provider: ObjectRef) -> bool {
        targets.iter().any(|bound| {
            bound.target == Target::Card(provider.card) && bound.object == Some(provider)
        })
    }

    pub(super) fn locked_target_life_costs(&self, a: &Announcement) -> Vec<u32> {
        let mut fees = Vec::new();
        if !a.item.targets.is_empty() {
            self.visit_opposing_target_life_costs(a.item.controller, |source, amount| {
                if Self::applies_target_life_cost(&a.item.targets, source) {
                    fees.push(amount);
                }
                false
            });
        }
        fees
    }

    pub(super) fn target_payment_announcement_fundable(
        &self,
        a: &Announcement,
        cost: ManaCost,
    ) -> bool {
        let Some(source) = a.item.source else {
            return false;
        };
        if let Some(index) = a.item.mode.map(usize::from)
            && (!self.item_mode_eligible(&a.item, index)
                || !self.teamwork_mode_allowed(&a.item, index)
                || !self.blight_mode_allowed(&a.item, index))
        {
            return false;
        }
        let cast = a.item.cast.expect("spell cast choices");
        cast.method != CastMethod::Normal
            || a.source_mana_alternative().is_some()
            || a.independent_additional_mana().is_some()
            || cast.flash_surcharge
            || self.spell_cost_option_available_for_definition(
                a.item.controller,
                source.id,
                a.cost_before_exile_play_surcharge(cost),
                source.card,
            )
    }

    fn completion_qualification(
        &self,
        c: &Completion<'_>,
        rule: TargetRule,
        prefer_matching: bool,
    ) -> Option<bool> {
        let _ = prefer_matching;
        let mut group = 0;
        while let Some(spec) = c.plan.group(group) {
            let count = c.chosen.iter().filter(|bound| bound.group == group).count()
                + usize::from(group == c.group && c.extra.is_some());
            if count > spec.max
                || (group < c.group || group == c.group && c.finished)
                    && count < usize::from(spec.min)
            {
                return None;
            }
            group += 1;
        }
        // Only declared targets affect the displayed provisional cost. No
        // hypothetical target is selected to obtain a discount or evade a tax.
        Some(
            c.chosen.iter().any(|bound| {
                self.same_target_object(*bound)
                    && self.cast_target_matches(rule, c.actor, c.source, c.definition, bound.target)
            }) || c.extra.is_some_and(|target| {
                self.cast_target_matches(rule, c.actor, c.source, c.definition, target)
            }),
        )
    }
    fn completion_modifiers(&self, c: &Completion<'_>) -> Option<(bool, bool)> {
        let reduced = match c.definition.target_cost_reduction() {
            Some(r) => self.completion_qualification(c, r.rule, true)?,
            None => false,
        };
        let increased = match c.definition.target_cost_increase() {
            Some(r) => self.completion_qualification(c, r.rule, false)?,
            None => false,
        };
        Some((reduced, increased))
    }
    pub(super) fn with_additional_target_mana(
        mut base: ManaCost,
        definition: Card,
        targets: usize,
    ) -> Option<ManaCost> {
        if let Some(mana) = definition.additional_target_mana() {
            let additional = u32::try_from(targets.saturating_sub(1)).ok()?;
            for (amount, extra) in base.colored.iter_mut().zip(mana.colored) {
                *amount = amount.checked_add(extra.checked_mul(additional)?)?;
            }
            base.generic = base
                .generic
                .checked_add(mana.generic.checked_mul(additional)?)?;
        }
        Some(base)
    }

    /// A proposal before targets are complete uses a legal completion; the
    /// final locked total uses the concrete bound target identities instead.
    pub(super) fn target_proposal_total(
        &self,
        a: &Announcement,
        base: ManaCost,
        plan: TargetPlan,
        extra: Option<Target>,
        finished: bool,
        reduction: u64,
    ) -> Option<ManaCost> {
        let source = a.item.source.expect("spell cost source");
        let c = Completion {
            actor: a.item.controller,
            source: a.item.target_source(),
            definition: source.card,
            plan,
            chosen: &a.item.targets,
            group: a.group,
            extra,
            finished,
        };
        let (reduced, increased) = self.completion_modifiers(&c)?;
        let target_count = c.chosen.len() + usize::from(c.extra.is_some());
        let base = Self::with_additional_target_mana(base, source.card, target_count)?;
        let x = u64::from(a.item.x.unwrap_or(0));
        let extra = (u64::from(a.payment_x_symbols()) * x).checked_add(
            source
                .card
                .target_cost_increase()
                .filter(|_| increased)
                .map_or(0, |r| u64::from(r.generic)),
        )?;
        let discount = source
            .card
            .target_cost_reduction()
            .filter(|_| reduced)
            .map_or(
                ManaCost {
                    colored: [0; 6],
                    generic: 0,
                },
                |r| r.discount,
            );
        self.spell_total_with_reductions_and_types(
            a.item.controller,
            source.id,
            base,
            source.card,
            source.card.fixed_mana_value() + u64::from(source.card.x_symbols()) * x,
            extra,
            discount,
            reduction,
            source.facts.granted_subtypes,
        )
    }
    fn target_total(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        qualifies: (bool, bool),
        extra_reduction: u64,
    ) -> Option<ManaCost> {
        let definition = self.effective_spell_card(id);
        let discount = definition
            .target_cost_reduction()
            .filter(|_| qualifies.0)
            .map_or(
                ManaCost {
                    colored: [0; 6],
                    generic: 0,
                },
                |rule| rule.discount,
            );
        self.spell_total_with_reductions(
            actor,
            id,
            base,
            definition,
            definition.fixed_mana_value(),
            definition
                .target_cost_increase()
                .filter(|_| qualifies.1)
                .map_or(0, |r| u64::from(r.generic)),
            discount,
            extra_reduction,
        )
    }
    pub(super) fn target_spell_cost(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        targets: &[BoundTarget],
    ) -> Option<ManaCost> {
        let extra = self
            .state
            .announcing
            .as_ref()
            .filter(|a| {
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id
                    == id
                    && a.item.controller == actor
            })
            .map_or(0, Self::announcement_optional_reduction);
        self.target_spell_cost_with_extra(actor, id, base, targets, extra)
    }
    pub(super) fn target_spell_cost_with_extra(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        targets: &[BoundTarget],
        extra: u64,
    ) -> Option<ManaCost> {
        let card = self.effective_spell_card(id);
        let reduced = card.target_cost_reduction().is_some_and(|r| {
            targets.iter().any(|b| {
                self.same_target_object(*b)
                    && self.cast_target_matches(r.rule, actor, self.object_ref(id), card, b.target)
            })
        });
        let increased = self.target_spell_increase(actor, id, card, targets);
        let base = Self::with_additional_target_mana(base, card, targets.len())?;
        self.target_total(actor, id, base, (reduced, increased != 0), extra)
    }
    pub(super) fn target_spell_increase(
        &self,
        actor: Player,
        id: CardId,
        card: Card,
        targets: &[BoundTarget],
    ) -> u64 {
        card.target_cost_increase()
            .filter(|r| {
                targets.iter().any(|b| {
                    self.same_target_object(*b)
                        && self.cast_target_matches(
                            r.rule,
                            actor,
                            self.object_ref(id),
                            card,
                            b.target,
                        )
                })
            })
            .map_or(0, |r| u64::from(r.generic))
    }
    pub(super) fn initial_target_cost_fundable(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> bool {
        // Costs that depend on targets are established after concrete targets.
        self.spell_cost_option_available(actor, id, base)
    }
    pub(super) fn announcement_target_fundable(
        &self,
        a: &Announcement,
        plan: Option<TargetPlan>,
        cost: Option<ManaCost>,
        offspring: Option<bool>,
        extra: Option<Target>,
        finished: bool,
    ) -> bool {
        let _ = (cost, offspring);
        if !self.exact_x_targets_fundable(a, extra, finished) {
            return false;
        }
        let plan = plan.unwrap_or(a.plan);
        let Some(spec) = plan.group(a.group) else {
            return extra.is_none();
        };
        let count = a
            .item
            .targets
            .iter()
            .filter(|bound| bound.group == a.group)
            .count()
            + usize::from(extra.is_some());
        if count > spec.max || finished && count < usize::from(spec.min) {
            return false;
        }
        extra.is_none_or(|target| {
            !a.item
                .targets
                .iter()
                .any(|bound| bound.group == a.group && bound.target == target)
                && self.item_target_matches(&a.item, spec.rule, target)
                && self.target_group_compatible(spec.rule, a.group, &a.item.targets, target)
        })
    }
    pub(super) fn provisional_target_cost(&self, a: &Announcement, base: ManaCost) -> ManaCost {
        let c = Completion {
            actor: a.item.controller,
            source: a.item.target_source(),
            definition: a.item.source.expect("cost source").card,
            plan: a.plan,
            chosen: &a.item.targets,
            group: a.group,
            extra: None,
            finished: false,
        };
        let qualifies = self
            .completion_modifiers(&c)
            .expect("completable target cost");
        self.target_total(
            a.item.controller,
            a.item
                .source
                .expect("cost announcement has object source")
                .id,
            base,
            qualifies,
            Self::announcement_optional_reduction(a),
        )
        .capacity("spell mana cost exhausted")
    }
}
