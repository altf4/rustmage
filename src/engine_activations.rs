//! Zone-qualified activations: legality, captured source context, and committed costs.
use super::*;

impl ActivatedAbility {
    pub(super) fn source_zone(&self) -> Zone {
        match self.origin {
            ActivationOrigin::Battlefield => Zone::Battlefield,
            ActivationOrigin::Hand => Zone::Hand,
            ActivationOrigin::Graveyard => Zone::Graveyard,
        }
    }
}

impl Game {
    pub(super) fn add_mana(&mut self, player: Player, production: Mana) {
        for (pool, amount) in self.state.players[player.index()]
            .mana
            .iter_mut()
            .zip(production)
        {
            *pool = pool
                .checked_add(amount)
                .capacity("mana representation exhausted");
        }
    }

    pub(super) fn activation_guard_available(&self, guard: ActivationGuard, actor: Player) -> bool {
        match guard {
            ActivationGuard::Always => true,
            ActivationGuard::OwnTurn => self.state.active == actor,
            ActivationGuard::ControlSubtypeAny(_) => self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield && o.owner == actor && guard.matches_card(o.card)
            }),
        }
    }
    pub(super) fn activation_cost_available(
        &self,
        card: CardId,
        ability: &ActivatedAbility,
        index: usize,
        actor: Player,
        sorcery: bool,
    ) -> bool {
        self.activation_guard_available(ability.guard, actor)
            && Self::object_activation_available(
                &self.state.objects[card as usize],
                self.keywords(card),
                self.state.players[actor.index()].life,
                ability,
                index,
                actor,
                sorcery,
            )
    }

    pub(super) fn object_activation_available(
        object: &Object,
        keywords: Keywords,
        life: i128,
        ability: &ActivatedAbility,
        index: usize,
        actor: Player,
        sorcery: bool,
    ) -> bool {
        if object.zone != ability.source_zone()
            || object.owner != actor
            || (ability.sorcery && !sorcery)
            || ((ability.once_per_turn || ability.exhaust)
                && object.used_abilities & (1u64 << index) != 0)
            || i128::from(ability.cost.life) > life
        {
            return false;
        }
        if ability.cost.tap
            && (object.tapped
                || (object.card.is_creature()
                    && object.summoning_sick
                    && !keywords.contains(Keywords::HASTE)))
        {
            return false;
        }
        if let Some(change) = ability.cost.loyalty
            && (!sorcery
                || object.used_abilities & object.card.loyalty_mask() != 0
                || (change < 0 && object.counters.loyalty < u64::from(change.unsigned_abs())))
        {
            return false;
        }
        if let Some((kind, amount)) = ability.cost.remove {
            let count = object.counters.available(kind);
            if count < u128::from(amount) {
                return false;
            }
        }
        true
    }

    pub(super) fn visit_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        for (index, ability) in object.card.activations().iter().enumerate() {
            if !self.activation_cost_available(card, ability, index, actor, sorcery) {
                continue;
            }
            let done = counter_payments(ability.cost, object.counters, |counter_payment| {
                let fundable = || {
                    ability.cost.mana_options().any(|cost| {
                        self.activation_can_fund(
                            FundingGoal {
                                reserve: None,
                                actor,
                                required: cost.colored,
                                generic: cost.generic,
                                source: Some((
                                    self.object_ref(card),
                                    index as u16,
                                    counter_payment,
                                    object.card,
                                )),
                            },
                            ability.cost.additional,
                        )
                    })
                };
                if self.targets_available(ability.targets, actor, self.object_ref(card))
                    && fundable()
                {
                    let done = if let Some(production) = ability.mana_production() {
                        production.visit(|production| {
                            emit(Action::AnnounceMana {
                                card,
                                ability: index as u16,
                                counter_payment,
                                production,
                            })
                        })
                    } else {
                        emit(Action::AnnounceActivate {
                            card,
                            ability: index as u16,
                            counter_payment,
                        })
                    };
                    if done {
                        return true;
                    }
                }
                // Direct actions have no owned external-cost selection.
                if ability.cost.additional.is_some() {
                    return false;
                }
                let mut done = false;
                let mut seen = std::collections::HashSet::new();
                for cost in ability.cost.mana_options() {
                    payments(
                        self.state.players[actor.index()].mana,
                        cost.colored,
                        cost.generic,
                        |payment| {
                            if done
                                || (!ability.cost.alternatives.is_empty() && !seen.insert(payment))
                            {
                                return;
                            }
                            if let Some(production) = ability.mana_production() {
                                done = production.visit(|production| {
                                    emit(Action::ActivateMana {
                                        card,
                                        ability: index as u16,
                                        payment,
                                        counter_payment,
                                        production,
                                    })
                                });
                                return;
                            }
                            let plan = ability.targets;
                            let Some(spec) = plan.single() else {
                                if self.targets_available(plan, actor, self.object_ref(card)) {
                                    done = emit(Action::BeginActivate {
                                        card,
                                        ability: index as u16,
                                        payment,
                                        counter_payment,
                                    });
                                }
                                return;
                            };
                            if spec.max > 1 || (spec.max > 0 && ability.cost.remove.is_some()) {
                                let mut count = 0;
                                self.visit_targets(spec.rule, actor, self.object_ref(card), |_| {
                                    count += 1;
                                    false
                                });
                                if count >= usize::from(spec.min) {
                                    done = emit(Action::BeginActivate {
                                        card,
                                        ability: index as u16,
                                        payment,
                                        counter_payment,
                                    });
                                }
                            } else {
                                if spec.min == 0 {
                                    done = emit(Action::Activate {
                                        card,
                                        ability: index as u16,
                                        target: None,
                                        payment,
                                        counter_payment,
                                    });
                                }
                                if !done && spec.max > 0 {
                                    done = self.visit_targets(
                                        spec.rule,
                                        actor,
                                        self.object_ref(card),
                                        |target| {
                                            emit(Action::Activate {
                                                card,
                                                ability: index as u16,
                                                target: Some(target),
                                                payment,
                                                counter_payment,
                                            })
                                        },
                                    );
                                }
                            }
                        },
                    );
                    if done {
                        break;
                    }
                }
                done
            });
            if done {
                return true;
            }
        }
        false
    }

    pub(super) fn activation_item(
        &mut self,
        actor: Player,
        card: CardId,
        ability: u16,
    ) -> SpellView {
        self.state.passes = 0;
        SpellView {
            id: self.next_stack_id(),
            kind: StackKind::ActivatedAbility { ability },
            source_card: self.state.objects[card as usize].card,
            source_keywords: self.keywords(card),
            source_tapped: self.state.objects[card as usize].tapped,
            source_facts: self.damage_facts(card),
            related_stack: None,
            damage_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_attachment: self.state.objects[card as usize].attached_to,
            card,
            object: self.object_ref(card),
            controller: actor,
            targets: Vec::new(),
            mode: None,
        }
    }

    pub(super) fn pay_activation(
        &mut self,
        item: &SpellView,
        payment: Mana,
        counter_payment: Counters,
    ) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            unreachable!()
        };
        let definition = &item.source_card.activations()[ability as usize];
        let cost: ActivationCost = definition.cost;
        for (pool, cost) in self.state.players[item.controller.index()]
            .mana
            .iter_mut()
            .zip(payment)
        {
            *pool -= cost;
        }
        self.lose_life(item.controller, i128::from(cost.life));
        let object = &mut self.state.objects[item.card as usize];
        if object.generation != item.object.generation || object.zone != definition.source_zone() {
            assert!(
                !cost.tap
                    && !cost.sacrifice
                    && !cost.discard
                    && !cost.exile_self
                    && cost.remove.is_none()
                    && cost.loyalty.is_none(),
                "unpaid source cost"
            );
            return;
        }
        object.used_abilities |= 1u64 << ability;
        if cost.tap {
            object.tapped = true;
        }
        object.counters.plus_one -= counter_payment.plus_one;
        object.counters.minus_one -= counter_payment.minus_one;
        object.counters.stun -= counter_payment.stun;
        object.counters.loyalty -= counter_payment.loyalty;
        if let Some(change) = cost.loyalty {
            if change < 0 {
                object.counters.loyalty -= u64::from(change.unsigned_abs());
            } else {
                object.counters.add(CounterKind::Loyalty, change as u64);
            }
        }
        if cost.exile_self {
            self.move_object(item.card, Zone::Exile);
        } else if cost.discard {
            self.discard_batch(&[item.card], definition.cycling.then_some(item.object));
        } else if cost.sacrifice {
            self.move_object(item.card, Zone::Graveyard);
        }
    }
}

/// Fixed small counter costs enumerate allocations, not subsets of individual counters.
pub(super) fn counter_payments(
    cost: ActivationCost,
    available: Counters,
    mut emit: impl FnMut(Counters) -> bool,
) -> bool {
    let Some((kind, amount)) = cost.remove else {
        return emit(Counters::default());
    };
    let amount = u64::from(amount);
    if let Some(kind) = kind {
        if available.get(kind) < amount {
            return false;
        }
        let mut paid = Counters::default();
        paid.add(kind, amount);
        return emit(paid);
    }
    fn visit(
        index: usize,
        left: u64,
        available: Counters,
        paid: Counters,
        emit: &mut impl FnMut(Counters) -> bool,
    ) -> bool {
        let kind = CounterKind::ALL[index];
        if index + 1 == CounterKind::ALL.len() {
            if left > available.get(kind) {
                return false;
            }
            let mut paid = paid;
            paid.add(kind, left);
            return emit(paid);
        }
        for count in 0..=left.min(available.get(kind)) {
            let mut paid = paid;
            paid.add(kind, count);
            if visit(index + 1, left - count, available, paid, emit) {
                return true;
            }
        }
        false
    }
    visit(0, amount, available, Counters::default(), &mut emit)
}
