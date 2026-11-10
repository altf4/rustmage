//! Target-dependent activation totals lock after targeting, before mana/payment.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// CR602.2b/601.2f: determine increases before reductions after the last
    /// target/X choice. Nothing in the subsequent payment window resamples it.
    pub(super) fn freeze_activation_total(&mut self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        if !a.pay_window || a.total_cost.is_some() {
            return false;
        }
        let source = a.item.source.expect("activation source");
        let increase =
            ability_world::CharacteristicWorld::named_activation_generic_increase(self, source.id);
        let discount = self.item_activation_discount(&a.item);
        let target_discount = if source.card.activation_target_color_discount(ability) != 0 {
            let target = a
                .item
                .targets
                .first()
                .expect("one mandatory color target")
                .target;
            self.activation_target_discount(
                a.item.controller,
                source.object,
                source.card,
                ability,
                target,
            )
        } else {
            self.selected_activation_target_discount(&a.item)
        };
        let [minimum, alternate] = a.activation_totals(
            a.raw_activation_choice(),
            discount.with_ordinary(target_discount),
            increase,
        );
        let mut total = minimum.expect("validated activation total");
        let order_costs = alternate.and_then(|alternate| {
            let mut candidate = a.clone();
            candidate.activation_generic_increase = increase;
            candidate.total_cost = Some(alternate);
            if !self.activation_total_fundable(&candidate) {
                return None;
            }
            candidate.total_cost = Some(total);
            if self.activation_total_fundable(&candidate) {
                Some([total, alternate])
            } else {
                total = alternate;
                None
            }
        });
        let a = self.state.announcing.as_mut().unwrap();
        a.activation_generic_increase = increase;
        a.activation_discount = u32::try_from(discount.ordinary).unwrap_or(u32::MAX);
        if let Some(totals) = order_costs {
            a.activation_reductions
                .get_or_insert_with(Box::default)
                .order_costs = Some(totals);
            self.state.pending = Pending::ActivationCost {
                player: a.item.controller,
            };
            true
        } else {
            a.total_cost = Some(total);
            false
        }
    }

    fn activation_target_totals(&self, a: &Announcement, target: Target) -> [Option<ManaCost>; 2] {
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            unreachable!()
        };
        let source = a.item.source.expect("activation source");
        let discount =
            self.item_activation_discount(&a.item)
                .with_ordinary(self.activation_target_discount(
                    a.item.controller,
                    source.object,
                    source.card,
                    ability,
                    target,
                ));
        a.activation_totals(
            a.raw_activation_choice(),
            discount,
            ability_world::CharacteristicWorld::named_activation_generic_increase(self, source.id),
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
        let discount = self.activation_target_discount(actor, source, definition, ability, target);
        base.generic = u32::try_from(u64::from(base.generic).saturating_sub(discount))
            .expect("a target discount cannot increase the cost");
        base
    }

    fn selected_activation_target_discount(&self, item: &SpellView) -> u64 {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            unreachable!("activation target discount");
        };
        let source = item.source.expect("activation source");
        let matching_target = source
            .card
            .activation_target_reduction(ability)
            .filter(|rule| {
                item.targets.iter().any(|b| {
                    self.same_target_object(*b)
                        && self.item_target_matches(item, rule.rule, b.target)
                })
            });
        let mut discount = matching_target.map_or(0, |rule| u64::from(rule.generic));
        for target in item.targets.iter().filter(|b| self.same_target_object(**b)) {
            if let (Some(kind), Target::Card(id)) = (
                source.card.activation_target_counter_discount(ability),
                target.target,
            ) && self.on_battlefield(id)
            {
                discount = discount.saturating_add(self.counter_count(id, kind));
            }
            discount = discount.saturating_add(u64::from(self.equip_target_reduction(
                item.controller,
                source.card,
                ability,
                target.target,
            )));
        }
        discount
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
        if proposal.declares_target_value_x() {
            return !self.target_value_x_intervals(&proposal).is_empty();
        }
        if source.card.activation_target_color_discount(ability) != 0
            || proposal.activation_discounts().leave_one != 0
                && Self::activation_target_sensitive(source.card, ability)
        {
            return self.activation_exact_target_fundable(&proposal, None, false);
        }
        let options = proposal.cost_options().count();
        (0..options).any(|slot| {
            proposal.select_cost(slot);
            if !Self::activation_target_sensitive(source.card, ability) {
                return proposal
                    .untargeted_activation_totals()
                    .into_iter()
                    .flatten()
                    .any(|total| {
                        let mut candidate = proposal.clone();
                        candidate.total_cost = Some(total);
                        self.activation_packet_fundable(&candidate)
                    });
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
        if ability.declares_target_value_x() {
            let mut packet = self.activation_packet(goal.actor, source.card, *index);
            packet.set_source_definition(*definition);
            packet.borrowed_slot = goal.source.as_ref().unwrap().4;
            let proposal = self.payment_announcement(packet, counters.clone(), None, None);
            return !self.target_value_x_intervals(&proposal).is_empty();
        }
        if definition.activation_target_color_discount(*index) != 0
            || Self::activation_target_sensitive(*definition, *index)
                && self
                    .activation_generic_discount(source.card, *definition, *index)
                    .leave_one
                    != 0
        {
            let mut packet = self.activation_packet(goal.actor, source.card, *index);
            packet.set_source_definition(*definition);
            packet.borrowed_slot = goal.source.as_ref().unwrap().4;
            let mut proposal = self.payment_announcement(packet, counters.clone(), None, None);
            proposal.plan = plan;
            let minimum = ability.minimum_x();
            if minimum != 0 {
                proposal.item.x = Some(minimum);
            }
            return self.activation_exact_target_fundable(&proposal, None, false);
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
            || a.activation_discounts().leave_one != 0
        {
            return self.activation_exact_target_fundable(a, extra, finished);
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

    fn activation_target_choice_fundable(&self, a: &Announcement, target: Target) -> bool {
        self.activation_target_totals(a, target)
            .into_iter()
            .flatten()
            .any(|total| {
                let mut selected = a.clone();
                selected.total_cost = Some(total);
                self.activation_packet_fundable(&selected)
            })
    }
    fn activation_exact_target_fundable(
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
                self.activation_target_choice_fundable(proposal, target)
            } else {
                self.visit_item_targets(
                    &a.item,
                    a.plan
                        .single()
                        .expect("one-target activation cost modifier")
                        .rule,
                    |target| self.activation_target_choice_fundable(proposal, target),
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
        if definition.activation_target_color_discount(ability) != 0
            || self
                .activation_generic_discount(card, definition, ability)
                .leave_one
                != 0
        {
            let packet = self.activation_packet(actor, card, ability);
            let mut proposal =
                self.payment_announcement(packet, counter_payment.clone(), None, None);
            let options = proposal.cost_options().count();
            return self.visit_targets(spec.rule, actor, source, |target| {
                let mut seen = std::collections::HashSet::new();
                for slot in 0..options {
                    proposal.select_cost(slot);
                    for total in self
                        .activation_target_totals(&proposal, target)
                        .into_iter()
                        .flatten()
                    {
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
                                if !done && seen.insert(payment) {
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
