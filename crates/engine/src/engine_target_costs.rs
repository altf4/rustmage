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
struct CostedTargetCompletion<'a> {
    item: &'a SpellView,
    plan: TargetPlan,
    optional_targets_help: bool,
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

    fn costed_target_completion(
        &self,
        completion: &CostedTargetCompletion<'_>,
        targets: &mut Vec<BoundTarget>,
        group: u16,
        closed: bool,
        first_candidate: usize,
        test: &mut impl FnMut(&[BoundTarget]) -> bool,
    ) -> bool {
        let Some(spec) = completion.plan.group(group) else {
            return test(targets);
        };
        let count = targets.iter().filter(|bound| bound.group == group).count();
        let min = self
            .exact_x_targets(completion.item)
            .unwrap_or(usize::from(spec.min));
        if count > spec.max {
            return false;
        }
        if count >= min
            && self.costed_target_completion(completion, targets, group + 1, false, 0, test)
        {
            return true;
        }
        if closed || count == spec.max || count >= min && !completion.optional_targets_help {
            return false;
        }
        let mut index = 0;
        self.visit_item_targets(completion.item, spec.rule, |target| {
            let candidate = index;
            index += 1;
            if candidate < first_candidate
                || targets
                    .iter()
                    .any(|b| b.group == group && b.target == target)
                || !self.target_group_compatible(spec.rule, group, targets, target)
            {
                return false;
            }
            let mut bound = self.bind_target(target);
            bound.group = group;
            targets.push(bound);
            let reachable =
                self.costed_target_completion(completion, targets, group, false, index, test);
            targets.pop();
            reachable
        })
    }

    pub(super) fn target_payment_fundable(
        &self,
        a: &Announcement,
        plan: TargetPlan,
        cost: ManaCost,
        offspring: Option<bool>,
        extra: Option<Target>,
        finished: bool,
    ) -> bool {
        let mut providers = Vec::new();
        self.visit_opposing_target_life_costs(a.item.controller, |source, amount| {
            providers.push((source, amount));
            false
        });
        let mut proposal = a.clone();
        proposal.cost = Some(cost);
        if let Some(offspring) = offspring {
            proposal.item.cast.as_mut().unwrap().offspring = Some(offspring);
        }
        let source = proposal.item.source.expect("spell cost source");
        let base = Self::announcement_base_cost(&proposal);
        let harmonizer = proposal.harmonizer.flatten();
        let discount = harmonizer
            .and_then(|reference| self.harmonizer_discount(&proposal.item, reference))
            .unwrap_or(0);
        let reduction = Self::announcement_optional_reduction(&proposal).saturating_add(discount);
        let mut targets = a.item.targets.clone();
        if let Some(target) = extra {
            let mut bound = self.bind_target(target);
            bound.group = a.group;
            targets.push(bound);
        }
        let completion = CostedTargetCompletion {
            item: &a.item,
            plan,
            optional_targets_help: source.card.target_cost_reduction().is_some(),
        };
        proposal.group = u16::MAX;
        self.costed_target_completion(
            &completion,
            &mut targets,
            a.group,
            finished,
            0,
            &mut |targets| {
                proposal.item.targets = targets.to_vec();
                let Some(total) =
                    self.target_proposal_total(&proposal, base, plan, None, true, reduction)
                else {
                    return false;
                };
                let mut choice = proposal.clone();
                // Install ordinary costs first so matching life amounts stay separate.
                choice.cast_costs.get_or_insert_with(Default::default);
                if let Some(threshold) = self.target_evidence_threshold(&choice) {
                    choice.lock_target_evidence(threshold);
                }
                choice.initialize_cast_costs(convoke::cost_potential(self, source.card));
                choice.add_target_life_costs(providers.iter().filter_map(|&(provider, amount)| {
                    Self::applies_target_life_cost(targets, provider).then_some(amount)
                }));
                self.harmonize_proposal_fundable(&choice, harmonizer, total)
            },
        )
    }

    pub(super) fn target_payment_announcement_fundable(
        &self,
        a: &Announcement,
        cost: ManaCost,
    ) -> bool {
        let source = a.item.source.expect("spell cost source");
        if a.needs_bargain()
            || a.needs_evidence()
            || a.needs_kicker()
            || a.needs_gift()
            || a.needs_blight()
        {
            return [false, true].into_iter().any(|pay| {
                let mut proposal = a.clone();
                let cast = proposal.item.cast.as_mut().unwrap();
                if a.needs_bargain() {
                    cast.bargain = Some(pay);
                } else if a.needs_evidence() {
                    cast.evidence = Some(pay);
                } else if a.needs_kicker() {
                    cast.kicker = Some(pay);
                } else if a.needs_gift() {
                    cast.gift = Some(if pay {
                        GiftDeclaration::Promised {
                            beneficiary: a.item.controller.other(),
                        }
                    } else {
                        GiftDeclaration::Declined
                    });
                } else {
                    cast.blight = Some(pay);
                }
                proposal.plan = self.target_spec(&proposal.item);
                self.target_payment_announcement_fundable(&proposal, cost)
            });
        }
        if a.item.mode.is_none()
            && let Some(modes) = a.item.modes()
        {
            return modes.iter().enumerate().any(|(index, _)| {
                if !self.item_mode_eligible(&a.item, index)
                    || !self.teamwork_mode_allowed(&a.item, index)
                    || !self.blight_mode_allowed(&a.item, index)
                {
                    return false;
                }
                let mut proposal = a.clone();
                proposal.item.mode = Some(u16::try_from(index).expect("mode index"));
                proposal.plan = self.target_spec(&proposal.item);
                Self::with_tiered_mana(cost, &proposal.item).is_some_and(|total| {
                    self.target_payment_announcement_fundable(&proposal, total)
                })
            });
        }
        if let Some(index) = a.item.mode.map(usize::from)
            && (!self.teamwork_mode_allowed(&a.item, index)
                || !self.blight_mode_allowed(&a.item, index))
        {
            return false;
        }
        let cast = a.item.cast.expect("spell cast choices");
        if cast.method == CastMethod::Normal
            && a.source_mana_alternative().is_none()
            && a.independent_additional_mana().is_none()
            && !cast.flash_surcharge
            && !self.spell_cost_option_available_for_definition(
                a.item.controller,
                source.id,
                a.cost_before_exile_play_surcharge(cost),
                source.card,
            )
        {
            return false;
        }
        if cast.method.is_harmonize() && a.harmonizer.is_none() {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            return self.harmonize_choice_fundable(&proposal, None)
                || (0..self.state.objects.len()).any(|id| {
                    self.harmonize_choice_fundable(&proposal, Some(self.object_ref(id as CardId)))
                });
        }
        let mut proposal = a.clone();
        proposal.cost = Some(cost);
        if proposal.forage.is_none()
            && let Some(spec) = proposal
                .additional_cost()
                .filter(|spec| forage::is_forage_cost(*spec))
        {
            return [
                ForageRoute::Food,
                ForageRoute::ExileThree,
                ForageRoute::Black,
            ]
            .into_iter()
            .any(|route| {
                if route == ForageRoute::Black && spec.kind != EffectChoiceKind::ForageOrBlack {
                    return false;
                }
                proposal.forage = Some(Box::new(forage::ForageSelection {
                    route: Some(route),
                    selected: Vec::new(),
                    skip_to: 0,
                }));
                self.announcement_target_fundable(&proposal, None, Some(cost), None, None, false)
            });
        }
        self.announcement_target_fundable(&proposal, None, Some(cost), None, None, false)
    }

    fn completion_qualification(
        &self,
        c: &Completion<'_>,
        rule: TargetRule,
        prefer_matching: bool,
    ) -> Option<bool> {
        let matches =
            |target| self.cast_target_matches(rule, c.actor, c.source, c.definition, target);
        let mut qualifies = c
            .chosen
            .iter()
            .any(|b| self.same_target_object(*b) && matches(b.target))
            || c.extra.is_some_and(matches);
        let mut group = 0;
        while let Some(spec) = c.plan.group(group) {
            let count = c.chosen.iter().filter(|b| b.group == group).count()
                + usize::from(group == c.group && c.extra.is_some());
            if count > spec.max {
                return None;
            }
            let closed = group < c.group || group == c.group && c.finished || count == spec.max;
            if closed {
                if count < usize::from(spec.min) {
                    return None;
                }
            } else {
                let mut available = 0;
                let mut can_qualify = false;
                let mut nonmatching = 0usize;
                self.visit_cast_targets(spec.rule, c.actor, c.source, c.definition, |target| {
                    if target == Target::Card(c.source.card)
                        || c.chosen
                            .iter()
                            .any(|b| b.group == group && b.target == target)
                        || group == c.group && c.extra == Some(target)
                    {
                        return false;
                    }
                    available += 1;
                    can_qualify |= matches(target);
                    nonmatching += usize::from(!matches(target));
                    false
                });
                if count + available < usize::from(spec.min) {
                    return None;
                }
                // A discount may choose a qualifying target. An increase is
                // unavoidable only when every legal completion includes one.
                qualifies |= if prefer_matching {
                    count < spec.max && can_qualify
                } else {
                    count + nonmatching < usize::from(spec.min)
                };
            }
            group += 1;
        }
        Some(qualifies)
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
        let source = self.object_ref(id);
        let card = self.effective_spell_card(id);
        let available = |plan| {
            let c = Completion {
                actor,
                source,
                definition: card,
                plan,
                chosen: &[],
                group: 0,
                extra: None,
                finished: false,
            };
            self.completion_modifiers(&c).is_some_and(|qualifies| {
                self.target_total(actor, id, base, qualifies, 0)
                    .is_some_and(|total| self.spell_can_fund_total(actor, id, total, None))
            })
        };
        match card.effect() {
            Effect::Modal(modes) => modes.iter().enumerate().any(|(index, mode)| {
                self.spell_mode_eligible(actor, source, card, index) && available(mode.targets)
            }),
            Effect::Spell(plan, _) | Effect::Aura(plan) => available(plan),
            _ => available(TargetPlan::NONE),
        }
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
        if !self.exact_x_targets_fundable(a, extra, finished) {
            return false;
        }
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && Self::activation_target_sensitive(
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                ability,
            )
        {
            return self.announcement_activation_target_fundable(a, extra, finished);
        }
        if a.item.kind != StackKind::Spell || !a.pay_window {
            return true;
        }
        let card = a.item.source.expect("spell cost source").card;
        let target_payment = self.cast_requires_target_payment(a.item.controller, card);
        if !target_payment && !card.has_target_cost_modifier() {
            return true;
        }
        if plan.is_none()
            && a.item.mode.is_none()
            && let Some(modes) = a.item.modes()
        {
            return modes.iter().enumerate().any(|(index, mode)| {
                self.item_mode_eligible(&a.item, index)
                    && self.announcement_target_fundable(
                        a,
                        Some(mode.targets),
                        cost,
                        offspring,
                        extra,
                        finished,
                    )
            });
        }
        let plan = plan.unwrap_or_else(|| {
            if a.plan == TargetPlan::NONE {
                self.target_spec(&a.item)
            } else {
                a.plan
            }
        });
        let test = |cost| {
            if target_payment {
                return self.target_payment_fundable(a, plan, cost, offspring, extra, finished);
            }
            let mut base = Self::with_offspring_cost(
                cost,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                offspring.unwrap_or_else(|| a.item.cast.is_some_and(|c| c.offspring == Some(true))),
            );
            base = Self::with_kicker_cost(
                base,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                a.item.cast.is_some_and(|c| c.kicker == Some(true)),
            );
            if a.forage
                .as_ref()
                .is_some_and(|f| f.route == Some(ForageRoute::Black))
            {
                base.colored[2] = base.colored[2]
                    .checked_add(1)
                    .capacity("forage alternative mana cost exhausted");
            }
            self.target_proposal_total(
                a,
                base,
                plan,
                extra,
                finished,
                Self::announcement_optional_reduction(a).saturating_add(
                    a.harmonizer
                        .flatten()
                        .and_then(|r| self.harmonizer_discount(&a.item, r))
                        .unwrap_or(0),
                ),
            )
            .is_some_and(|total| {
                if a.tap_payment().is_some() {
                    return self.waterbend_spell_total_fundable(a, total);
                }
                if a.independent_additional_mana().is_some() {
                    let mut proposal = a.clone();
                    proposal.cast_costs.get_or_insert_with(Default::default);
                    return self.harmonize_proposal_fundable(
                        &proposal,
                        a.harmonizer.flatten(),
                        total,
                    );
                }
                if a.item.cast.is_some_and(|c| c.method.is_harmonize())
                    || a.has_ordered_origin_payment()
                    || a.has_ordered_life_cost()
                {
                    return self.harmonize_proposal_fundable(a, a.harmonizer.flatten(), total);
                }
                self.spell_can_fund_total(
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .id,
                    total,
                    a.forage.as_ref().and_then(|f| f.route),
                )
            })
        };
        if let Some(cost) = cost.or(a.cost) {
            test(cost)
        } else {
            a.cost_options().any(test)
        }
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
        .expect("validated provisional cost")
    }
}
