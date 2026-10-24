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
    pub(super) fn activation_generic_discount(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
    ) -> u32 {
        definition
            .activation_counter_discount(ability)
            .map_or(0, |kind| {
                u32::try_from(self.counter_count(source, kind)).unwrap_or(u32::MAX)
            })
    }
    pub(super) fn item_activation_discount(&self, item: &SpellView) -> u32 {
        match item.kind {
            StackKind::ActivatedAbility { ability } => {
                self.activation_generic_discount(item.card, item.source_card, ability)
            }
            _ => 0,
        }
    }
    pub(super) fn spend_mana(&mut self, actor: Player, payment: Mana) {
        self.spend_mana_for(actor, payment, mana_lots::Purpose::Other);
    }
    /// Payment origins have no independent effects in the admitted subset.
    /// Spend red that expires sooner first, retaining the longer-lived subset.
    pub(super) fn spend_ordinary_mana(&mut self, actor: Player, payment: Mana) {
        let player = &mut self.state.players[actor.index()];
        let ordinary_red = player.mana[3] - player.combat_red_mana;
        let retained_paid = payment[3].saturating_sub(ordinary_red);
        player.combat_red_mana -= retained_paid;
        for (pool, paid) in player.mana.iter_mut().zip(payment) {
            *pool -= paid;
        }
    }

    pub(super) fn add_combat_red(&mut self, actor: Player, amount: u128) {
        let amount = u32::try_from(amount).capacity("combat mana representation exhausted");
        let player = &mut self.state.players[actor.index()];
        let red = player.mana[3]
            .checked_add(amount)
            .capacity("mana representation exhausted");
        let retained = player
            .combat_red_mana
            .checked_add(amount)
            .capacity("combat mana representation exhausted");
        player.mana[3] = red;
        player.combat_red_mana = retained;
    }

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
            ActivationGuard::CastInstantSorceryThisTurn => {
                self.state.players[actor.index()].instant_sorcery_cast_this_turn > 0
            }
            ActivationGuard::Always => true,
            ActivationGuard::HandSizeAtLeast(minimum) => {
                self.hand_count(actor) >= usize::from(minimum)
            }
            ActivationGuard::LifeAboveStarting(extra) => {
                self.state.players[actor.index()].life >= STARTING_LIFE + i128::from(extra)
            }
            ActivationGuard::OwnTurn => self.state.active == actor,
            ActivationGuard::ControlLandsAtLeast(minimum) => {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == actor
                            && self.current_kinds(*id as CardId) & 2 != 0
                    })
                    .count()
                    >= usize::from(minimum)
            }
            ActivationGuard::ControlSubtypeAny(_) => {
                self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == actor
                        && guard
                            .subtypes()
                            .iter()
                            .any(|s| self.has_subtype(id as CardId, s))
                })
            }
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
        !ability_world::CharacteristicWorld::activation_prohibited(self, actor, card)
            && self.printed_abilities_available(card)
            && (ability.mana_production().is_some() || !self.named_activation_prohibited(card))
            && self.activation_guard_available(ability.guard, actor)
            && Self::object_activation_available_with_kinds(
                &self.state.objects[card as usize],
                self.counter_amounts(card),
                self.current_kinds(card),
                || self.keywords(card),
                self.state.players[actor.index()].life,
                ability,
                index,
                actor,
                sorcery,
            )
    }

    // Shared real/virtual readiness query: no allocation or speculative state clone.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn object_activation_available_with_kinds(
        object: &Object,
        counters: CounterAmounts<'_>,
        kinds: u16,
        keywords: impl FnOnce() -> Keywords,
        life: i128,
        ability: &ActivatedAbility,
        index: usize,
        actor: Player,
        sorcery: bool,
    ) -> bool {
        if object.zone != ability.source_zone()
            || object.controller != actor
            || (ability.sorcery && !sorcery)
            || ((ability.once_per_turn || ability.exhaust)
                && object.used_abilities & (1u64 << index) != 0)
            || (ability.cost.life != 0 && i128::from(ability.cost.life) > life)
        {
            return false;
        }
        if ability.cost.tap
            && (object.tapped
                || (kinds & 1 != 0
                    && object.summoning_sick
                    && !keywords().contains(Keywords::HASTE)))
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
            let count = counters.available(kind);
            if count < u128::from(amount) {
                return false;
            }
        }
        true
    }

    pub(super) fn visit_granted_mana(
        &self,
        card: CardId,
        actor: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if ability_world::CharacteristicWorld::activation_prohibited(self, actor, card)
            || object.zone != Zone::Battlefield
            || object.controller != actor
            || object.tapped
            || (self.is_creature(card)
                && object.summoning_sick
                && !self.keywords(card).contains(Keywords::HASTE))
        {
            return false;
        }
        for grant in ability_world::CharacteristicWorld::granted_mana(self, card) {
            if grant.production.in_world(self, card).visit(|production| {
                emit(Action::TapGrantedMana {
                    card,
                    grantor: grant.source,
                    production,
                })
            }) {
                return true;
            }
        }
        false
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
            if object.card.grants_activation(index as u16) {
                continue;
            }
            if !self.activation_cost_available(card, ability, index, actor, sorcery) {
                continue;
            }
            if let Some(minimum) = ability.crew_minimum() {
                if self.crew_fundable(self.object_ref(card), actor, minimum)
                    && emit(Action::AnnounceActivate {
                        card,
                        ability: index as u16,
                        counter_payment: Counters::default(),
                    })
                {
                    return true;
                }
                continue;
            }
            if let Some(minimum) = ability.saddle_minimum() {
                if self.saddle_fundable(self.object_ref(card), actor, minimum)
                    && emit(Action::AnnounceActivate {
                        card,
                        ability: index as u16,
                        counter_payment: Counters::default(),
                    })
                {
                    return true;
                }
                continue;
            }
            let done = counter_payments(
                ability.cost,
                self.counter_amounts(card),
                |counter_payment| {
                    let fundable = || {
                        ability.cost.mana_options().any(|cost| {
                            self.activation_can_fund(
                                FundingGoal {
                                    reserve: None,
                                    actor,
                                    required: cost.colored,
                                    generic: cost.generic.saturating_sub(
                                        self.activation_generic_discount(
                                            card,
                                            object.card,
                                            index as u16,
                                        ),
                                    ),
                                    source: Some((
                                        self.object_ref(card),
                                        index as u16,
                                        counter_payment.clone(),
                                        object.card,
                                        None,
                                    )),
                                },
                                ability.cost.additional,
                            )
                        })
                    };
                    let modes = match ability.program.first() {
                        Some(Instruction::Modal(modes)) => Some(*modes),
                        _ => None,
                    };
                    let targets_available = modes.map_or_else(
                        || self.targets_available(ability.targets, actor, self.object_ref(card)),
                        |modes| {
                            modes.iter().any(|mode| {
                                self.targets_available(mode.targets, actor, self.object_ref(card))
                            })
                        },
                    );
                    if targets_available && fundable() {
                        let done = if let Some(production) = ability.mana_production() {
                            production.in_world(self, card).visit(|production| {
                                emit(Action::AnnounceMana {
                                    card,
                                    ability: index as u16,
                                    counter_payment: counter_payment.clone(),
                                    production,
                                })
                            })
                        } else {
                            emit(Action::AnnounceActivate {
                                card,
                                ability: index as u16,
                                counter_payment: counter_payment.clone(),
                            })
                        };
                        if done {
                            return true;
                        }
                    }
                    // A modal activation must choose its mode before targets and
                    // costs. Legacy direct actions cannot encode that declaration.
                    if modes.is_some() || ability.has_x_cost() {
                        return false;
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
                            cost.generic
                                .saturating_sub(self.activation_generic_discount(
                                    card,
                                    object.card,
                                    index as u16,
                                )),
                            |payment| {
                                if done
                                    || (!ability.cost.alternatives.is_empty()
                                        && !seen.insert(payment))
                                {
                                    return;
                                }
                                if let Some(production) = ability.mana_production() {
                                    done = production.in_world(self, card).visit(|production| {
                                        emit(Action::ActivateMana {
                                            card,
                                            ability: index as u16,
                                            payment,
                                            counter_payment: counter_payment.clone(),
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
                                            counter_payment: counter_payment.clone(),
                                        });
                                    }
                                    return;
                                };
                                if spec.max > 1 || (spec.max > 0 && ability.cost.remove.is_some()) {
                                    let mut count = 0;
                                    self.visit_targets(
                                        spec.rule,
                                        actor,
                                        self.object_ref(card),
                                        |_| {
                                            count += 1;
                                            false
                                        },
                                    );
                                    if count >= usize::from(spec.min) {
                                        done = emit(Action::BeginActivate {
                                            card,
                                            ability: index as u16,
                                            payment,
                                            counter_payment: counter_payment.clone(),
                                        });
                                    }
                                } else {
                                    if spec.min == 0 {
                                        done = emit(Action::Activate {
                                            card,
                                            ability: index as u16,
                                            target: None,
                                            payment,
                                            counter_payment: counter_payment.clone(),
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
                                                    counter_payment: counter_payment.clone(),
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
                },
            );
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
            source_move: None,
            borrowed_slot: None,
            target_controller_lki: None,
            gift: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            copy_event: None,
            target_allocations: None,
            captured_artifact_copy: None,
            source_copy_recipe: self
                .copy_recipe(self.object_ref(card))
                .filter(|r| r.exceptions != CopyExceptions::NONE)
                .map(Box::new),
            mana_copy_sources: None,
            is_copy: false,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            attachment_trigger: None,
            cast: None,
            id: self.next_stack_id(),
            kind: StackKind::ActivatedAbility { ability },
            source_card: self.state.objects[card as usize].card,
            source_keywords: self.keywords(card),
            source_tapped: self.state.objects[card as usize].tapped,
            source_facts: self.damage_facts(card),
            related_stack: None,
            entering_object: None,
            evidence_collected: false,
            damage_event: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_lki: self.state.objects[card as usize]
                .card
                .activations()
                .get(usize::from(ability))
                .and_then(|ability| {
                    card_quantities::counter_quantity_facts(ability.program, |kind| {
                        self.source_counter_quantity_live(self.object_ref(card), kind)
                            .expect("live activated source")
                    })
                }),
            sacrificed_toughness: None,
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
        item: &mut SpellView,
        payment: Mana,
        counter_payment: Counters,
    ) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            unreachable!()
        };
        let definition = &item.source_card.activation(ability);
        self.pay_activation_definition(item, payment, counter_payment, ability, definition);
    }

    // The same committed payment routine handles both ordinary authored abilities
    // and bounded typed fixtures that exercise ordered combinations of costs.
    pub(super) fn pay_activation_definition(
        &mut self,
        item: &mut SpellView,
        payment: Mana,
        counter_payment: Counters,
        ability: u16,
        definition: &ActivatedAbility,
    ) {
        let cost: ActivationCost = definition.cost;
        self.spend_mana_for(item.controller, payment, self.purpose_for_item(item));
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
        if let Some(slot) = item.borrowed_slot {
            self.record_borrowed_use(item.object, slot, definition.exhaust);
        } else {
            object.used_abilities |= 1u64 << ability;
        }
        self.pay_counters(item.card, &counter_payment);
        let object = &mut self.state.objects[item.card as usize];
        if let Some(change) = cost.loyalty {
            if change < 0 {
                object.counters.loyalty -= u64::from(change.unsigned_abs());
            } else {
                self.add_counter(item.card, CounterKind::Loyalty, change as u64);
            }
        }
        // Capture the tap event before a subsequent sacrifice/discard/exile
        // changes the source incarnation. All other paid source costs are set.
        if cost.tap {
            self.tap_objects(&[item.card]);
            if let Some(grant) = &item.attachment_trigger
                && item.source_card.grants_activation(ability)
            {
                self.tap_objects(&[grant.grantor.card]);
            }
        }
        let StackKind::ActivatedAbility {
            ability: definition_index,
        } = item.kind
        else {
            unreachable!()
        };
        if let Some((kind, amount)) = item.source_card.activation_put_counter(definition_index) {
            self.activation_cost_counters(item.object, kind, amount, item.controller);
        }
        self.refresh_local_source_counter_quantities(item);
        if cost.exile_self {
            self.move_object(item.card, Zone::Exile);
        } else if cost.discard {
            self.discard_batch(&[item.card], definition.cycling.then_some(item.object));
        } else if cost.sacrifice {
            self.sacrifice_batch(&[(item.controller, item.card)]);
        }
    }
}

/// Fixed small counter costs enumerate allocations, not subsets of individual counters.
pub(super) fn counter_payments<'a>(
    cost: ActivationCost,
    available: impl Into<CounterAmounts<'a>>,
    mut emit: impl FnMut(Counters) -> bool,
) -> bool {
    let available = available.into();
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
        available: CounterAmounts<'_>,
        paid: &mut Counters,
        emit: &mut impl FnMut(Counters) -> bool,
    ) -> bool {
        let kind = CounterKind::ALL[index];
        if index + 1 == CounterKind::ALL.len() {
            if left > available.get(kind) {
                return false;
            }
            paid.add(kind, left);
            let done = emit(paid.clone());
            paid.remove(kind, left);
            return done;
        }
        for count in 0..=left.min(available.get(kind)) {
            paid.add(kind, count);
            let done = visit(index + 1, left - count, available, paid, emit);
            paid.remove(kind, count);
            if done {
                return true;
            }
        }
        false
    }
    visit(0, amount, available, &mut Counters::default(), &mut emit)
}
