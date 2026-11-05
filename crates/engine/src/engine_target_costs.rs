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
    fn completion_qualification(
        &self,
        c: &Completion<'_>,
        rule: TargetRule,
        prefer_matching: bool,
    ) -> Option<bool> {
        let matches = |target| self.cast_target_matches(rule, c.actor, c.source, target);
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
                self.visit_cast_targets(spec.rule, c.actor, c.source, |target| {
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
        self.spell_total_with_reductions(
            a.item.controller,
            source.id,
            base,
            source.card,
            source.card.fixed_mana_value() + u64::from(source.card.x_symbols()) * x,
            extra,
            discount,
            reduction,
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
        let reduced = self
            .effective_spell_card(id)
            .target_cost_reduction()
            .is_some_and(|r| {
                targets.iter().any(|b| {
                    self.same_target_object(*b)
                        && self.cast_target_matches(r.rule, actor, self.object_ref(id), b.target)
                })
            });
        let increased =
            self.target_spell_increase(actor, id, self.effective_spell_card(id), targets);
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
                        && self.cast_target_matches(r.rule, actor, self.object_ref(id), b.target)
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
            Effect::Modal(modes) => modes.iter().any(|m| available(m.targets)),
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
        if a.item.kind != StackKind::Spell
            || !a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_target_cost_modifier()
        {
            return true;
        }
        if plan.is_none()
            && a.item.mode.is_none()
            && let Some(modes) = a.item.modes()
        {
            return modes.iter().any(|m| {
                self.announcement_target_fundable(
                    a,
                    Some(m.targets),
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
                if a.item.cast.is_some_and(|c| c.method.is_harmonize()) {
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
