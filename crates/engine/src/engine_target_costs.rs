//! Target-aware cost proposals. Queries observe current facts; virtual funding
//! never changes which targets qualify. Only the final total enters mutable state.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct Completion<'a> {
    actor: Player,
    source: ObjectRef,
    plan: TargetPlan,
    chosen: &'a [BoundTarget],
    group: u16,
    extra: Option<Target>,
    finished: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn completion_discount(&self, c: &Completion<'_>) -> Option<bool> {
        let reduction = self.state.objects[c.source.card as usize]
            .card
            .target_cost_reduction()?;
        let matches = |target| self.cast_target_matches(reduction.rule, c.actor, c.source, target);
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
                    false
                });
                if count + available < usize::from(spec.min) {
                    return None;
                }
                qualifies |= count < spec.max && can_qualify;
            }
            group += 1;
        }
        Some(qualifies)
    }
    fn target_total(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        qualifies: bool,
        extra_reduction: u64,
    ) -> Option<ManaCost> {
        let definition = self.effective_spell_card(id);
        let discount = definition
            .target_cost_reduction()
            .filter(|_| qualifies)
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
            0,
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
        let qualifies = self
            .effective_spell_card(id)
            .target_cost_reduction()
            .is_some_and(|r| {
                targets.iter().any(|b| {
                    self.same_target_object(*b)
                        && self.cast_target_matches(r.rule, actor, self.object_ref(id), b.target)
                })
            });
        self.target_total(actor, id, base, qualifies, extra)
    }
    pub(super) fn initial_target_cost_fundable(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> bool {
        let source = self.object_ref(id);
        let card = self.state.objects[id as usize].card;
        let available = |plan| {
            let c = Completion {
                actor,
                source,
                plan,
                chosen: &[],
                group: 0,
                extra: None,
                finished: false,
            };
            self.completion_discount(&c).is_some_and(|qualifies| {
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
            || a.item
                .source
                .expect("cost announcement has object source")
                .card
                .target_cost_reduction()
                .is_none()
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
        let plan = plan.unwrap_or(a.plan);
        let c = Completion {
            actor: a.item.controller,
            source: a.item.target_source(),
            plan,
            chosen: &a.item.targets,
            group: a.group,
            extra,
            finished,
        };
        let Some(qualifies) = self.completion_discount(&c) else {
            return false;
        };
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
            .is_some_and(|total| {
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
            plan: a.plan,
            chosen: &a.item.targets,
            group: a.group,
            extra: None,
            finished: false,
        };
        let qualifies = self.completion_discount(&c) == Some(true);
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
