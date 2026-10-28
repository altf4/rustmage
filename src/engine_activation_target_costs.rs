//! Target-dependent activation totals lock after targeting, before mana/payment.
use super::*;
impl Game {
    fn activation_target_total(
        &self,
        actor: Player,
        source: ObjectRef,
        definition: Card,
        ability: u16,
        target: Target,
        mut base: ManaCost,
    ) -> ManaCost {
        if let Some(rule) = definition.activation_target_reduction(ability)
            && self.cast_target_matches(rule.rule, actor, source, target)
        {
            base.generic = base.generic.saturating_sub(rule.generic);
        }
        if let (Some(kind), Target::Card(id)) = (
            definition.activation_target_counter_discount(ability),
            target,
        ) && self.on_battlefield(id)
        {
            base.generic = base
                .generic
                .saturating_sub(u32::try_from(self.counter_count(id, kind)).unwrap_or(u32::MAX));
        }
        base.generic = base
            .generic
            .saturating_sub(self.equip_target_reduction(actor, definition, ability, target));
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
        for target in item.targets.iter().filter(|b| self.same_target_object(**b)) {
            if let (Some(kind), Target::Card(id)) = (
                item.source_card.activation_target_counter_discount(ability),
                target.target,
            ) && self.on_battlefield(id)
            {
                base.generic = base.generic.saturating_sub(
                    u32::try_from(self.counter_count(id, kind)).unwrap_or(u32::MAX),
                );
            }
            base.generic = base.generic.saturating_sub(self.equip_target_reduction(
                item.controller,
                item.source_card,
                ability,
                target.target,
            ));
        }
        base
    }
    pub(super) fn initial_activation_target_fundable(
        &self,
        goal: FundingGoal,
        plan: TargetPlan,
    ) -> bool {
        let Some((source, index, counters, definition, _)) = goal.source.as_ref() else {
            unreachable!("activation source");
        };
        let ability = definition.activation(*index);
        if *counters == Counters::default() && self.first_equip_zero_available(goal.actor, ability)
        {
            return true;
        }
        if !Self::activation_target_sensitive(*definition, *index) {
            return self.activation_can_fund(goal, ability.cost.additional);
        }
        let spec = plan.single().expect("one-target activation cost modifier");
        self.visit_targets(spec.rule, goal.actor, *source, |target| {
            let total = self.activation_target_total(
                goal.actor,
                *source,
                *definition,
                *index,
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
        // Before the optional declaration, at least the captured zero route is fundable.
        if a.equip_zero_available && a.cost.is_none() {
            return true;
        }
        let test = |base: ManaCost| {
            let free = a.activation_cost_route == ActivationCostRoute::FirstEquipZero;
            let goal = FundingGoal {
                actor: a.item.controller,
                required: base.colored,
                generic: base.generic,
                reserve: None,
                source: (!free).then(|| {
                    (
                        a.item.object,
                        ability,
                        a.counter_payment.clone(),
                        a.item.source_card,
                        a.item.borrowed_slot,
                    )
                }),
            };
            if free {
                self.can_fund_for(goal, self.purpose_for_item(&a.item))
            } else {
                self.activation_can_fund(goal, a.additional_cost())
            }
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
        let fund_target = |target| {
            let apply = |base| {
                test(self.activation_target_total(
                    a.item.controller,
                    a.item.target_source(),
                    a.item.source_card,
                    ability,
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
                .expect("one-target activation cost modifier");
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
        if definition.activation(ability).cost.additional.is_some() {
            return false;
        }
        let source = self.object_ref(card);
        let spec = definition
            .activation(ability)
            .targets
            .single()
            .expect("one-target activation cost modifier");
        self.visit_targets(spec.rule, actor, source, |target| {
            let mut done = false;
            for base in self.activation_mana_costs(card, definition, ability, 0) {
                let total =
                    self.activation_target_total(actor, source, definition, ability, target, base);
                if !self.activation_can_fund(
                    FundingGoal {
                        actor,
                        required: total.colored,
                        generic: total.generic,
                        reserve: None,
                        source: Some((source, ability, counter_payment.clone(), definition, None)),
                    },
                    definition.activation(ability).cost.additional,
                ) {
                    continue;
                }
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
