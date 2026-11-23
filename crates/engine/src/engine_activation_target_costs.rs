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
        if a.additional_cost()
            .is_some_and(|c| c.kind.uses_target_mana_value())
        {
            let Target::Card(target) = a.item.targets[0].target else {
                unreachable!("mana-value exile cost has one card target");
            };
            let minimum = self.object_mana_value(target);
            self.state
                .announcing
                .as_mut()
                .unwrap()
                .cast_costs
                .as_mut()
                .expect("ordered mana-value exile cost")
                .target_mana_value = Some(minimum);
        }
        let a = self.state.announcing.as_ref().unwrap();
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
        let total = minimum.capacity("activation mana cost exhausted");
        // Both legal reduction orders remain declarations even when one cannot
        // be funded. The caller chooses; the total then locks.
        let order_costs = alternate.map(|alternate| [total, alternate]);
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
        let proposal =
            self.payment_announcement(packet.clone(), counter_payment.clone(), None, None);
        self.activation_packet_admissible(&proposal)
            && (proposal.declares_target_value_x()
                || self.item_targets_available(packet, proposal.plan)
                || packet.modes().is_some())
    }
    pub(super) fn initial_activation_target_fundable(
        &self,
        goal: FundingGoal,
        plan: TargetPlan,
    ) -> bool {
        let Some((source, ability, counters, definition, borrowed_slot)) = goal.source else {
            return false;
        };
        if self.state.objects.get(source.card as usize).is_none()
            || self.object_ref(source.card) != source
        {
            return false;
        }
        let rules = definition.activation(ability);
        // This fixed zero-mana cost has no mana window or other cost group
        // that could remove the current prohibition. Taxes, X and replacement
        // equip routes still declare normally and validate their actual payment.
        if goal.generic == 0
            && goal.required == [0; 6]
            && rules.cost
                == (ActivationCost {
                    tap: rules.cost.tap,
                    ..ActivationCost::ZERO
                })
            && definition.activation_x_symbols(ability) == 0
            && !rules.is_equip()
            && activations::printed_counter_placement_prohibited(self, source, definition, ability)
        {
            return false;
        }
        let mut packet = self.activation_packet(goal.actor, source.card, ability);
        packet.set_source_definition(definition);
        packet.borrowed_slot = borrowed_slot;
        let mut proposal = self.payment_announcement(packet, counters, None, None);
        proposal.plan = plan;
        self.activation_packet_admissible(&proposal)
            && (proposal.declares_target_value_x()
                || self.item_targets_available(&proposal.item, plan)
                || proposal.item.modes().is_some())
    }
    pub(super) fn announcement_activation_target_fundable(
        &self,
        a: &Announcement,
        extra: Option<Target>,
        _finished: bool,
    ) -> bool {
        // The targeting state machine enforces group counts and joint target
        // constraints. Check a supplied target, never any possible continuation.
        extra.is_none_or(|target| self.item_target_matches(&a.item, a.spec.rule, target))
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
                        if !self.activation_packet_admissible(&selected) {
                            continue;
                        }
                        let mut done = false;
                        let payment_cost =
                            cast_origins::activation_payment_cost(self, actor, source, total);
                        payments(
                            self.state.players[actor.index()].mana,
                            payment_cost.colored,
                            payment_cost.generic,
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
                let payment_cost =
                    cast_origins::activation_payment_cost(self, actor, source, total);
                payments(
                    self.state.players[actor.index()].mana,
                    payment_cost.colored,
                    payment_cost.generic,
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
