//! Target-dependent activation totals lock after targeting, before mana/payment.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// CR602.2b/601.2f: determine increases before reductions after the last
    /// target/X choice. Nothing in the subsequent payment window resamples it.
    pub(super) fn freeze_activation_total(&mut self) {
        let Some(a) = self.state.announcing.as_ref() else {
            return;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return;
        };
        if !a.pay_window || a.total_cost.is_some() {
            return;
        }
        let source = a.item.source.expect("activation source");
        if source.card.activation_target_color_discount(ability) != 0 {
            let target = a
                .item
                .targets
                .first()
                .expect("one mandatory color target")
                .target;
            let total = self
                .activation_color_target_total(a, target)
                .expect("validated activation total");
            let increase = ability_world::CharacteristicWorld::named_activation_generic_increase(
                self, source.id,
            );
            let discount = self.item_activation_discount(&a.item);
            let a = self.state.announcing.as_mut().unwrap();
            a.activation_generic_increase = increase;
            a.activation_discount = discount;
            a.total_cost = Some(total);
            return;
        }
        let choice = a.raw_activation_choice();
        let increase =
            ability_world::CharacteristicWorld::named_activation_generic_increase(self, source.id);
        let discount = self.item_activation_discount(&a.item);
        let x = if choice.route == ActivationCostRoute::Printed {
            u64::from(a.item.x.unwrap_or(0)) * u64::from(source.card.activation_x_symbols(ability))
        } else {
            0
        };
        let base = power_up::reduce(
            choice.base,
            choice.reduction,
            discount,
            x + u64::from(increase),
        )
        .expect("validated activation total");
        let total = self.selected_activation_target_cost(&a.item, base);
        let a = self.state.announcing.as_mut().unwrap();
        a.activation_generic_increase = increase;
        a.activation_discount = discount;
        a.total_cost = Some(total);
    }

    fn activation_color_target_total(&self, a: &Announcement, target: Target) -> Option<ManaCost> {
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            unreachable!()
        };
        let source = a.item.source.expect("activation source");
        let choice = a.raw_activation_choice();
        let discount = u64::from(self.item_activation_discount(&a.item)).saturating_add(
            self.activation_target_discount(
                a.item.controller,
                source.object,
                source.card,
                ability,
                target,
            ),
        );
        let x = if choice.route == ActivationCostRoute::Printed {
            u64::from(a.item.x.unwrap_or(0)) * u64::from(source.card.activation_x_symbols(ability))
        } else {
            0
        };
        power_up::reduce_wide(
            choice.base,
            choice.reduction,
            discount,
            x + u64::from(
                ability_world::CharacteristicWorld::named_activation_generic_increase(
                    self, source.id,
                ),
            ),
        )
    }

    fn activation_target_discount(
        &self,
        actor: Player,
        source: ObjectRef,
        definition: Card,
        ability: u16,
        target: Target,
    ) -> u64 {
        let mut reduction = definition
            .activation_target_reduction(ability)
            .filter(|rule| self.activation_target_matches(rule.rule, actor, source, target))
            .map_or(0, |rule| u64::from(rule.generic));
        if let Target::Card(id) = target
            && self.on_battlefield(id)
        {
            if let Some(kind) = definition.activation_target_counter_discount(ability) {
                reduction = reduction.saturating_add(self.counter_count(id, kind));
            }
            let factor = definition.activation_target_color_discount(ability);
            if factor != 0 {
                reduction = reduction.saturating_add(
                    u64::from(factor)
                        * u64::from(
                            (ability_world::CharacteristicWorld::colors(self, id) & 31)
                                .count_ones(),
                        ),
                );
            }
        }
        reduction.saturating_add(u64::from(
            self.equip_target_reduction(actor, definition, ability, target),
        ))
    }

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
            && self.activation_target_matches(rule.rule, actor, source, target)
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
        if definition.activation_target_color_discount(ability) != 0
            && let Target::Card(id) = target
            && self.on_battlefield(id)
        {
            let colors = (ability_world::CharacteristicWorld::colors(self, id) & 31).count_ones();
            let reduction =
                u64::from(definition.activation_target_color_discount(ability)) * u64::from(colors);
            base.generic = u32::try_from(u64::from(base.generic).saturating_sub(reduction))
                .expect("a target discount cannot increase the cost");
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
        let Some(source) = item.source else {
            return base;
        };
        if let Some(rule) = source.card.activation_target_reduction(ability)
            && item.targets.iter().any(|b| {
                self.same_target_object(*b) && self.item_target_matches(item, rule.rule, b.target)
            })
        {
            base.generic = base.generic.saturating_sub(rule.generic);
        }
        for target in item.targets.iter().filter(|b| self.same_target_object(**b)) {
            if source.card.activation_target_color_discount(ability) != 0
                && let Target::Card(id) = target.target
                && self.on_battlefield(id)
            {
                let colors =
                    (ability_world::CharacteristicWorld::colors(self, id) & 31).count_ones();
                let reduction = u64::from(source.card.activation_target_color_discount(ability))
                    * u64::from(colors);
                base.generic = u32::try_from(u64::from(base.generic).saturating_sub(reduction))
                    .expect("a target discount cannot increase the cost");
            }
            if let (Some(kind), Target::Card(id)) = (
                source.card.activation_target_counter_discount(ability),
                target.target,
            ) && self.on_battlefield(id)
            {
                base.generic = base.generic.saturating_sub(
                    u32::try_from(self.counter_count(id, kind)).unwrap_or(u32::MAX),
                );
            }
            base.generic = base.generic.saturating_sub(self.equip_target_reduction(
                item.controller,
                source.card,
                ability,
                target.target,
            ));
        }
        base
    }
    /// Keep acquired activation identity in the same payment context used by
    /// announcement. A printed slot at the same index has independent history.
    pub(super) fn initial_activation_packet_target_fundable(
        &self,
        packet: &SpellView,
        counter_payment: &Counters,
    ) -> bool {
        let StackKind::ActivatedAbility { ability } = packet.kind else {
            unreachable!("activated packet");
        };
        let source = packet.source.expect("activated packet source");
        let mut proposal =
            self.payment_announcement(packet.clone(), counter_payment.clone(), None, None);
        if proposal.nonmana_x() {
            return !self.nonmana_x_intervals(&proposal).is_empty();
        }
        if source.card.activation_target_color_discount(ability) != 0 {
            return self.activation_color_target_fundable(&proposal, None, false);
        }
        let options = proposal.cost_options().count();
        (0..options).any(|slot| {
            proposal.select_cost(slot);
            if !Self::activation_target_sensitive(source.card, ability) {
                return self.activation_packet_fundable(&proposal);
            }
            let spec = proposal
                .plan
                .single()
                .expect("one-target activation cost modifier");
            self.visit_item_targets(packet, spec.rule, |target| {
                let mut candidate = proposal.clone();
                candidate.item.targets.push(self.bind_target(target));
                candidate.total_cost = Some(self.activation_target_total(
                    packet.controller,
                    source.object,
                    source.card,
                    ability,
                    target,
                    proposal.cost.expect("selected activation cost"),
                ));
                self.activation_packet_fundable(&candidate)
            })
        })
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
        if ability.nonmana_x() {
            let mut packet = self.activation_packet(goal.actor, source.card, *index);
            packet.set_source_definition(*definition);
            packet.borrowed_slot = goal.source.as_ref().unwrap().4;
            let proposal = self.payment_announcement(packet, counters.clone(), None, None);
            return !self.nonmana_x_intervals(&proposal).is_empty();
        }
        if definition.activation_target_color_discount(*index) != 0 {
            let mut packet = self.activation_packet(goal.actor, source.card, *index);
            packet.set_source_definition(*definition);
            packet.borrowed_slot = goal.source.as_ref().unwrap().4;
            let mut proposal = self.payment_announcement(packet, counters.clone(), None, None);
            proposal.plan = plan;
            let minimum = ability.minimum_x();
            if minimum != 0 {
                proposal.item.x = Some(minimum);
            }
            return self.activation_color_target_fundable(&proposal, None, false);
        }
        if *counters == Counters::default() && self.first_equip_zero_available(goal.actor, ability)
        {
            let mut packet = self.activation_packet(goal.actor, source.card, *index);
            packet.set_source_definition(*definition);
            let announcement = self.payment_announcement(packet, counters.clone(), None, None);
            let purpose = self.purpose_for_item(&announcement.item);
            for (slot, base) in announcement.cost_options().enumerate() {
                if announcement.activation_route_for_index(slot)
                    != ActivationCostRoute::FirstEquipZero
                {
                    continue;
                }
                let fund = |total: ManaCost| {
                    self.can_fund_for(
                        FundingGoal {
                            source: None,
                            reserve: None,
                            actor: goal.actor,
                            required: total.colored,
                            generic: total.generic,
                        },
                        purpose,
                    )
                };
                if !Self::activation_target_sensitive(*definition, *index) {
                    if fund(base) {
                        return true;
                    }
                } else {
                    let spec = plan.single().expect("one-target activation cost modifier");
                    if self.visit_targets(spec.rule, goal.actor, *source, |target| {
                        fund(self.activation_target_total(
                            goal.actor,
                            *source,
                            *definition,
                            *index,
                            target,
                            base,
                        ))
                    }) {
                        return true;
                    }
                }
            }
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
        if a.item
            .source
            .expect("activation source")
            .card
            .activation_target_color_discount(ability)
            != 0
        {
            return self.activation_color_target_fundable(a, extra, finished);
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
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .object,
                        ability,
                        a.counter_payment.clone(),
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .card,
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
                test(
                    self.activation_target_total(
                        a.item.controller,
                        a.item.target_source(),
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .card,
                        ability,
                        target,
                        base,
                    ),
                )
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

    fn activation_color_choice_fundable(&self, a: &Announcement, target: Target) -> bool {
        self.activation_color_target_total(a, target)
            .is_some_and(|total| {
                let mut selected = a.clone();
                selected.total_cost = Some(total);
                self.activation_packet_fundable(&selected)
            })
    }
    fn activation_color_target_fundable(
        &self,
        a: &Announcement,
        extra: Option<Target>,
        finished: bool,
    ) -> bool {
        if a.total_cost.is_some() {
            return self.activation_packet_fundable(a);
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
        let fund_choice = |proposal: &Announcement| {
            if let Some(target) = chosen {
                self.activation_color_choice_fundable(proposal, target)
            } else {
                self.visit_item_targets(
                    &a.item,
                    a.plan.single().expect("one color target").rule,
                    |target| self.activation_color_choice_fundable(proposal, target),
                )
            }
        };
        if a.activation_choice.is_some() {
            return fund_choice(a);
        }
        let mut proposal = a.clone();
        let options = a.cost_options().count();
        (0..options).any(|slot| {
            proposal.select_cost(slot);
            fund_choice(&proposal)
        })
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
        if definition.activation_target_color_discount(ability) != 0 {
            let packet = self.activation_packet(actor, card, ability);
            let mut proposal =
                self.payment_announcement(packet, counter_payment.clone(), None, None);
            let options = proposal.cost_options().count();
            return self.visit_targets(spec.rule, actor, source, |target| {
                for slot in 0..options {
                    proposal.select_cost(slot);
                    let Some(total) = self.activation_color_target_total(&proposal, target) else {
                        continue;
                    };
                    let mut selected = proposal.clone();
                    selected.total_cost = Some(total);
                    if !self.activation_packet_fundable(&selected) {
                        continue;
                    }
                    let mut done = false;
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
                        return true;
                    }
                }
                false
            });
        }
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
