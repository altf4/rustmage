//! One chosen target can reduce an activated slot's generic cost. The total is
//! fixed after targeting and before any mana ability/payment changes that target.
use super::*;
impl Game {
    fn activation_target_total(
        &self,
        actor: Player,
        source: ObjectRef,
        rule: crate::cards::TargetActivationReduction,
        target: Target,
        mut base: ManaCost,
    ) -> ManaCost {
        if self.cast_target_matches(rule.rule, actor, source, target) {
            base.generic = base.generic.saturating_sub(rule.generic);
        }
        base
    }
    pub(super) fn selected_activation_target_cost(
        &self,
        item: &SpellView,
        mut base: ManaCost,
    ) -> ManaCost {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return base;
        };
        if let Some(rule) = item.source_card.activation_target_reduction(ability)
            && item.targets.iter().any(|b| {
                self.same_target_object(*b) && self.item_target_matches(item, rule.rule, b.target)
            })
        {
            base.generic = base.generic.saturating_sub(rule.generic);
        }
        base
    }
    pub(super) fn initial_activation_target_fundable(
        &self,
        goal: FundingGoal,
        plan: TargetPlan,
    ) -> bool {
        let Some((source, index, _, definition, _)) = goal.source.as_ref() else {
            unreachable!("activation source");
        };
        let ability = definition.activation(*index);
        let Some(rule) = definition.activation_target_reduction(*index) else {
            return self.activation_can_fund(goal, ability.cost.additional);
        };
        let spec = plan
            .single()
            .expect("checked one-target activation reduction");
        self.visit_targets(spec.rule, goal.actor, *source, |target| {
            let total = self.activation_target_total(
                goal.actor,
                *source,
                rule,
                target,
                ManaCost {
                    colored: goal.required,
                    generic: goal.generic,
                },
            );
            let mut proposal = goal.clone();
            proposal.required = total.colored;
            proposal.generic = total.generic;
            self.activation_can_fund(proposal, ability.cost.additional)
        })
    }
    pub(super) fn announcement_activation_target_fundable(
        &self,
        a: &Announcement,
        extra: Option<Target>,
        finished: bool,
    ) -> bool {
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            unreachable!();
        };
        let test = |base: ManaCost| {
            self.activation_can_fund(
                FundingGoal {
                    actor: a.item.controller,
                    required: base.colored,
                    generic: base.generic,
                    reserve: None,
                    source: Some((
                        a.item.object,
                        ability,
                        a.counter_payment.clone(),
                        a.item.source_card,
                        a.item.borrowed_slot,
                    )),
                },
                a.additional_cost(),
            )
        };
        if let Some(total) = a.total_cost {
            return test(total);
        }
        let chosen = extra.or_else(|| {
            a.item
                .targets
                .first()
                .filter(|b| self.same_target_object(**b))
                .map(|b| b.target)
        });
        if finished && chosen.is_none() {
            return false;
        }
        let rule = a
            .item
            .source_card
            .activation_target_reduction(ability)
            .unwrap();
        let fund_target = |target| {
            let apply = |base| {
                test(self.activation_target_total(
                    a.item.controller,
                    a.item.target_source(),
                    rule,
                    target,
                    base,
                ))
            };
            if let Some(base) = a.cost {
                apply(base)
            } else {
                a.cost_options().any(apply)
            }
        };
        if let Some(target) = chosen {
            fund_target(target)
        } else {
            let spec = a
                .plan
                .single()
                .expect("checked one-target activation reduction");
            self.visit_item_targets(&a.item, spec.rule, fund_target)
        }
    }
    pub(super) fn visit_target_cost_direct_actions(
        &self,
        card: CardId,
        ability: u16,
        actor: Player,
        counter_payment: &Counters,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let definition = self.state.objects[card as usize].card;
        let source = self.object_ref(card);
        let rule = definition.activation_target_reduction(ability).unwrap();
        let spec = definition
            .activation(ability)
            .targets
            .single()
            .expect("checked one-target reduction");
        self.visit_targets(spec.rule, actor, source, |target| {
            let mut done = false;
            for base in self.activation_mana_costs(card, definition, ability, 0) {
                let total = self.activation_target_total(actor, source, rule, target, base);
                payments(
                    self.state.players[actor.index()].mana,
                    total.colored,
                    total.generic,
                    |payment| {
                        if !done {
                            done = emit(Action::Activate {
                                card,
                                ability,
                                target: Some(target),
                                payment,
                                counter_payment: counter_payment.clone(),
                            });
                        }
                    },
                );
                if done {
                    break;
                }
            }
            done
        })
    }
}
