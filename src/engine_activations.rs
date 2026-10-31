//! Zone-qualified activations: legality, captured source context, and committed costs.
use super::*;

/// The first unpaid phase after a counter-placement replacement batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActivationPaymentPhase {
    AfterLoyalty,
    AfterSourceCounter,
}

/// Caller-specific work that must happen only after every activation cost.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ActivationPaymentCompletion {
    Announcement,
    DirectStack,
    DirectMana {
        actor: Player,
        card: CardId,
        production: Mana,
        quality: Option<mana_lots::Quality>,
    },
    AlreadyOnStack,
    Crew(CrewContext),
    Saddle(Vec<ObjectRef>),
}

/// Owned only while a counter-placement cost is waiting on replacements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActivationPayment {
    pub item: SpellView,
    pub ability: u16,
    pub definition: ActivatedAbility,
    pub phase: ActivationPaymentPhase,
    pub completion: ActivationPaymentCompletion,
}

impl ActivatedAbility {
    pub(super) fn source_zone(&self) -> Zone {
        match self.origin {
            ActivationOrigin::Battlefield => Zone::Battlefield,
            ActivationOrigin::Hand => Zone::Hand,
            ActivationOrigin::Graveyard => Zone::Graveyard,
        }
    }
}

/// A pure loyalty activation has no mana-payment window in which to remove a
/// counter prohibition. Composite costs need their funding world evaluated at
/// the eventual payment point instead of applying this early restriction.
pub(super) fn pure_loyalty_placement_prohibited<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    ability: &ActivatedAbility,
) -> bool {
    !ability.cost.mana_component && simple_loyalty_placement_prohibited(world, source, ability)
}

/// Whether this simple mana-bearing loyalty cost cannot currently place its
/// counters. Funding must re-evaluate this in each projected payment world.
pub(super) fn mana_loyalty_placement_prohibited<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    ability: &ActivatedAbility,
) -> bool {
    ability.cost.mana_component && simple_loyalty_placement_prohibited(world, source, ability)
}

fn simple_loyalty_placement_prohibited<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    ability: &ActivatedAbility,
) -> bool {
    ability.cost.loyalty.is_some_and(|change| change > 0)
        && ability.cost.additional.is_none()
        && !ability.cost.tap
        && !ability.cost.sacrifice
        && !ability.cost.exile_self
        && !ability.cost.discard
        && ability.cost.life == 0
        && ability.cost.remove.is_none()
        && replacements::counters_prohibited(world, source)
}

/// Starting restriction reads the same object overlay in actual and funding worlds.
pub(super) fn graveyard_cards_at_least<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    kind: u16,
    minimum: u16,
) -> bool {
    world
        .game()
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, _)| {
            let id = *id as CardId;
            let object = world.object(id);
            object.zone == Zone::Graveyard
                && object.owner == actor
                && object.is_card()
                && (kind == 0 || world.kinds(id) & kind != 0)
        })
        .take(usize::from(minimum))
        .count()
        >= usize::from(minimum)
}
/// Starting guard: a payment may later remove the witness without invalidating activation.
pub(super) fn controls_tough_creature<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    minimum: u16,
) -> bool {
    let query = characteristic_query::CharacteristicQuery::in_world(world);
    (0..world.game().state.objects.len()).any(|id| {
        let id = id as CardId;
        let object = world.object(id);
        object.zone == Zone::Battlefield
            && object.controller == actor
            && world.kinds(id) & 1 != 0
            && query.stats(id).toughness >= i128::from(minimum)
    })
}
impl Game {
    /// Retain the exact ability and caller while replacement choices are made.
    pub(super) fn suspend_activation_payment(
        &mut self,
        item: SpellView,
        phase: ActivationPaymentPhase,
        route: ActivationCostRoute,
        completion: ActivationPaymentCompletion,
    ) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            unreachable!("only activations pay activation costs")
        };
        let source = item.source.expect("activated ability has an object source");
        let mut definition = *source.card.activation(ability);
        if route == ActivationCostRoute::FirstEquipZero {
            assert!(definition.is_equip());
            definition.cost = ActivationCost::ZERO;
        }
        assert!(self.state.activation_payment.is_none());
        self.state.activation_payment = Some(Box::new(ActivationPayment {
            item,
            ability,
            definition,
            phase,
            completion,
        }));
    }

    /// Resume after a replacement batch has completely drained. Returns true
    /// when the next cost staged another replacement batch.
    pub(super) fn resume_activation_payment(&mut self) -> bool {
        let Some(mut payment) = self.state.activation_payment.take() else {
            return false;
        };
        if let Some(phase) = self.resume_activation_costs(
            &mut payment.item,
            payment.ability,
            &payment.definition,
            payment.phase,
        ) {
            payment.phase = phase;
            self.state.activation_payment = Some(payment);
            return true;
        }
        match payment.completion {
            ActivationPaymentCompletion::Announcement => {
                self.resume_activation_announcement(payment.item);
            }
            ActivationPaymentCompletion::DirectStack => {
                self.finish_direct_activation_on_stack(payment.item);
                self.state.passes = 0;
            }
            ActivationPaymentCompletion::DirectMana {
                actor,
                card,
                production,
                quality,
            } => {
                self.add_quality_mana(actor, production, quality);
                self.produce_mountain_tap_bonus(actor, payment.item.mana_tap_bonus.unwrap_or(0));
                self.committed_mana_reflexive(&payment.item);
                self.reopen_evidence_after_mana_sacrifice(card);
            }
            ActivationPaymentCompletion::AlreadyOnStack => {
                self.finish_direct_activation_on_stack(payment.item);
            }
            ActivationPaymentCompletion::Crew(context) => {
                self.finish_paid_crew(payment.item, context);
            }
            ActivationPaymentCompletion::Saddle(selected) => {
                self.finish_paid_saddle(payment.item, selected);
            }
        }
        self.state.replacements.is_some()
    }
    pub(super) fn activation_generic_discount(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
    ) -> u32 {
        let subtype_discount =
            definition
                .activation_subtype_discount(ability)
                .map_or(0, |subtype| {
                    let controller = self.state.objects[source as usize].controller;
                    u32::try_from(
                        self.state
                            .objects
                            .iter()
                            .enumerate()
                            .filter(|(id, object)| {
                                object.zone == Zone::Battlefield
                                    && object.controller == controller
                                    && self.has_subtype(*id as CardId, subtype)
                            })
                            .count(),
                    )
                    .unwrap_or(u32::MAX)
                });
        self.power_up_generic_discount(source, definition, ability)
            .saturating_add(subtype_discount)
            .saturating_add(
                definition
                    .activation_counter_discount(ability)
                    .map_or(0, |kind| {
                        u32::try_from(self.counter_count(source, kind)).unwrap_or(u32::MAX)
                    }),
            )
    }
    pub(super) fn item_activation_discount(&self, item: &SpellView) -> u32 {
        match item.kind {
            StackKind::ActivatedAbility { ability } => item.source.map_or(0, |source| {
                self.activation_generic_discount(source.id, source.card, ability)
            }),
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

    pub(super) fn activation_guard_available(
        &self,
        guard: ActivationGuard,
        source: CardId,
        actor: Player,
    ) -> bool {
        match guard {
            ActivationGuard::ClassLevelExactly(level) => {
                self.state.objects[source as usize].class_level == level
            }
            ActivationGuard::CastInstantSorceryThisTurn => {
                self.state.players[actor.index()].instant_sorcery_cast_this_turn > 0
            }
            ActivationGuard::ScriedOrSurveilledThisTurn => {
                self.state.players[actor.index()].scried_or_surveilled_this_turn
            }
            ActivationGuard::OpponentNoncombatDamageThisTurn => {
                self.state.players[actor.other().index()].noncombat_damage_this_turn
            }
            ActivationGuard::SourceSolved => self.case_solved(self.object_ref(source)),
            ActivationGuard::Always => true,
            ActivationGuard::SourceAttackedThisTurn => self
                .state
                .attacked_this_turn
                .contains(&self.object_ref(source)),
            ActivationGuard::ControlCreatureToughnessAtLeast(minimum) => {
                activations::controls_tough_creature(self, actor, minimum)
            }
            ActivationGuard::GraveyardCardsAtLeast { kind, minimum } => {
                graveyard_cards_at_least(self, actor, kind, minimum)
            }
            ActivationGuard::SourceEnteredOrControlsBasicLand => {
                self.entered_this_turn(self.object_ref(source))
                    || ability_world::CharacteristicWorld::controls_basic_land(self, actor)
            }
            ActivationGuard::HandSizeAtLeast(minimum) => {
                self.hand_count(actor) >= usize::from(minimum)
            }
            ActivationGuard::LifeAboveStarting(extra) => {
                self.state.players[actor.index()].life >= STARTING_LIFE + i128::from(extra)
            }
            ActivationGuard::OwnTurn => self.state.active == actor,
            ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype, minimum } => {
                let world = self.characteristic_query();
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.controller == actor
                            && ability_world::CharacteristicWorld::kinds(&world, *id as CardId) & 64
                                != 0
                            && ability_world::CharacteristicWorld::subtypes(&world, *id as CardId)
                                .has(object.card, subtype)
                    })
                    .fold(0u64, |sum, (id, _)| {
                        sum.saturating_add(ability_world::CharacteristicWorld::counter_count(
                            &world,
                            id as CardId,
                            CounterKind::Loyalty,
                        ))
                    })
                    >= minimum
            }
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
    /// Loyalty timing permission follows current controller, type, and subtype.
    pub(super) fn loyalty_timing(
        &self,
        card: CardId,
        ability: &ActivatedAbility,
        actor: Player,
        sorcery: bool,
    ) -> bool {
        sorcery
            || (ability.cost.loyalty.is_some()
                && self.state.players[actor.index()].jace_loyalty_anytime
                && self.state.objects[card as usize].controller == actor
                && self.current_kinds(card) & 64 != 0
                && self.has_subtype(card, "Jace"))
    }

    pub(super) fn activation_cost_available(
        &self,
        card: CardId,
        ability: &ActivatedAbility,
        index: usize,
        actor: Player,
        sorcery: bool,
    ) -> bool {
        let readiness = self.activation_readiness(actor, ability);
        !pure_loyalty_placement_prohibited(self, self.object_ref(card), &readiness)
            && !ability_world::CharacteristicWorld::activation_prohibited(self, actor, card)
            && (!ability.cost.sacrifice
                || !ability_world::CharacteristicWorld::sacrifice_prohibited(self, card))
            && self.printed_abilities_available(card)
            && (ability.mana_production().is_some()
                || (!self.nonmana_actions_prohibited() && !self.named_activation_prohibited(card)))
            && self.activation_guard_available(ability.guard, card, actor)
            && self.power_up_available(
                self.object_ref(card),
                self.state.objects[card as usize].card,
                index as u16,
                None,
            )
            && Self::object_activation_available_with_kinds(
                &self.state.objects[card as usize],
                self.counter_amounts(card),
                self.current_kinds(card),
                || ability_world::CharacteristicWorld::activation_keywords(self, card),
                self.state.players[actor.index()].life,
                &readiness,
                index,
                actor,
                self.loyalty_timing(card, ability, actor, sorcery),
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
                || object.loyalty_activated_this_turn
                || (change < 0 && object.counters.loyalty < u64::from(change.unsigned_abs())))
        {
            return false;
        }
        if let Some((kind, amount)) = ability.cost.remove {
            let loyalty_reserve = ability
                .cost
                .loyalty
                .filter(|change| *change < 0)
                .map_or(0, |change| u64::from(change.unsigned_abs()));
            let count = counters.available(kind).saturating_sub(
                if kind.is_none() || kind == Some(CounterKind::Loyalty) {
                    u128::from(loyalty_reserve)
                } else {
                    0
                },
            );
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
                && !ability_world::CharacteristicWorld::activation_keywords(self, card)
                    .contains(Keywords::HASTE))
        {
            return false;
        }
        for grant in ability_world::CharacteristicWorld::granted_mana(self, card) {
            if grant.sacrifice
                && ability_world::CharacteristicWorld::sacrifice_prohibited(self, card)
            {
                continue;
            }
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
            if object.card.legacy_subtype_mana_activation(index as u16)
                || object.card.grants_activation(index as u16)
                || object.card.self_granted_activation_template(index as u16)
            {
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
            let done = self.visit_activation_counter_payments(
                actor,
                ability,
                self.counter_amounts(card),
                |counter_payment| {
                    let fundable = || {
                        self.activation_mana_costs(card, object.card, index as u16, 0)
                            .any(|cost| {
                                self.initial_activation_target_fundable(
                                    FundingGoal {
                                        reserve: None,
                                        actor,
                                        required: cost.colored,
                                        generic: cost.generic,
                                        source: Some((
                                            self.object_ref(card),
                                            index as u16,
                                            counter_payment.clone(),
                                            object.card,
                                            None,
                                        )),
                                    },
                                    ability.targets,
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
                    // A direct action cannot declare which optional whole-cost
                    // route it pays; expose the explicit announcement instead.
                    if self.first_equip_zero_available(actor, ability) {
                        return false;
                    }
                    if mana_loyalty_placement_prohibited(self, self.object_ref(card), ability) {
                        return false;
                    }
                    if object
                        .card
                        .activation_target_reduction(index as u16)
                        .is_some()
                        || object
                            .card
                            .activation_target_counter_discount(index as u16)
                            .is_some()
                        || self.active_target_equip_reducer(actor, object.card, index as u16)
                    {
                        return self.visit_target_cost_direct_actions(
                            card,
                            index as u16,
                            actor,
                            &counter_payment,
                            emit,
                        );
                    }
                    // A modal activation must choose its mode before targets and
                    // costs. Legacy direct actions cannot encode that declaration.
                    if modes.is_some()
                        || ability.has_x_cost()
                        || ability.source_counter_cost().is_some()
                    {
                        return false;
                    }
                    // Direct actions have no owned external-cost selection.
                    if ability.cost.additional.is_some() {
                        return false;
                    }
                    let mut done = false;
                    let mut seen = std::collections::HashSet::new();
                    for cost in self.activation_mana_costs(card, object.card, index as u16, 0) {
                        payments(
                            self.state.players[actor.index()].mana,
                            cost.colored,
                            cost.generic,
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
            source: Some(StackSourceView {
                id: card,
                object: self.object_ref(card),
                card: self.state.objects[card as usize].card,
                facts: self.damage_facts(card),
            }),
            source_less_rule: None,
            additional_triggers: 0,
            source_move: None,
            borrowed_slot: None,
            self_granted_activation: None,
            target_controller_lki: None,
            crime_qualifying_target: false,
            target_kinds_lki: None,
            target_type_slots: None,
            gift: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            target_allocations: None,
            captured_artifact_copy: None,
            source_copy_recipe: self
                .copy_recipe(self.object_ref(card))
                .filter(|r| {
                    r.exceptions != CopyExceptions::NONE
                        || r.overrides.is_some()
                        || r.faces.is_some()
                })
                .map(Box::new),
            source_ability_origin: self.copy_ability_origin(self.object_ref(card)),
            mana_copy_sources: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            loyalty_x: None,
            reflexive_target_limit: None,
            attachment_trigger: None,
            cast: None,
            id: self.next_stack_id(),
            kind: StackKind::ActivatedAbility { ability },
            source_transform: self.state.objects[card as usize]
                .card
                .activations()
                .get(usize::from(ability))
                .is_some_and(|ability| ability.program.contains(&Instruction::TransformSource))
                .then(|| self.transform_serial(self.object_ref(card))),

            split_second: false,
            source_keywords: self.keywords(card),
            source_tapped: self.state.objects[card as usize].tapped,
            mana_tap_bonus: None,
            related_stack: None,
            entering_object: None,
            evidence_collected: false,
            damage_event: None,
            damage_received: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_payment: None,
            loyalty_counters_paid: 0,
            exiled_symbol_cards: None,
            exiled_trigger_cards: None,
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
            source_quantity_lki: None,
            event_player: None,
            event_object: None,
            manifest_event: None,
            sacrificed_toughness: None,
            sacrificed_power: None,
            source_attachment: self.state.objects[card as usize].attached_to,

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
    ) -> Option<ActivationPaymentPhase> {
        self.pay_activation_route(item, payment, counter_payment, ActivationCostRoute::Printed)
    }
    pub(super) fn pay_activation_route(
        &mut self,
        item: &mut SpellView,
        payment: Mana,
        counter_payment: Counters,
        route: ActivationCostRoute,
    ) -> Option<ActivationPaymentPhase> {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            unreachable!()
        };
        let source = item.source.expect("activated ability has an object source");
        let mut definition = *source.card.activation(ability);
        if route == ActivationCostRoute::FirstEquipZero {
            assert!(definition.is_equip());
            assert_eq!(counter_payment, Counters::default());
            definition.cost = ActivationCost::ZERO;
        }
        self.pay_activation_definition(item, payment, counter_payment, ability, &definition)
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
    ) -> Option<ActivationPaymentPhase> {
        if !self.start_activation_payment(item, payment, counter_payment, definition) {
            return None;
        }
        self.pay_activation_loyalty(item, definition);
        if self.state.replacements.is_some() {
            return Some(ActivationPaymentPhase::AfterLoyalty);
        }
        self.pay_activation_tap(item, ability, definition);
        self.pay_activation_source_counter(item);
        if self.state.replacements.is_some() {
            return Some(ActivationPaymentPhase::AfterSourceCounter);
        }
        self.finish_activation_source_costs(item, definition);
        None
    }

    /// Continue only the unpaid source-cost phases after a replacement batch.
    pub(super) fn resume_activation_costs(
        &mut self,
        item: &mut SpellView,
        ability: u16,
        definition: &ActivatedAbility,
        phase: ActivationPaymentPhase,
    ) -> Option<ActivationPaymentPhase> {
        if phase == ActivationPaymentPhase::AfterLoyalty {
            self.pay_activation_tap(item, ability, definition);
            self.pay_activation_source_counter(item);
            if self.state.replacements.is_some() {
                return Some(ActivationPaymentPhase::AfterSourceCounter);
            }
        }
        self.finish_activation_source_costs(item, definition);
        None
    }

    /// Apply the receipt and removal phases exactly once, before placements.
    fn start_activation_payment(
        &mut self,
        item: &mut SpellView,
        payment: Mana,
        counter_payment: Counters,
        definition: &ActivatedAbility,
    ) -> bool {
        let source = item.source.expect("activated ability has an object source");
        let cost: ActivationCost = definition.cost;
        self.record_cast_mana_activation(item, payment, &counter_payment);
        self.spend_mana_for(item.controller, payment, self.purpose_for_item(item));
        self.pay_life(item.controller, i128::from(cost.life));
        let object = &mut self.state.objects[source.id as usize];
        if object.generation != source.object.generation || object.zone != definition.source_zone()
        {
            assert!(
                !cost.tap
                    && !cost.sacrifice
                    && !cost.discard
                    && !cost.exile_self
                    && cost.remove.is_none()
                    && cost.loyalty.is_none(),
                "unpaid source cost"
            );
            return false;
        }
        if let Some(change) = cost.loyalty {
            item.loyalty_counters_paid = item
                .loyalty_counters_paid
                .checked_add(counter_payment.get(CounterKind::Loyalty))
                .and_then(|paid| {
                    paid.checked_add(if change < 0 {
                        u64::from(change.unsigned_abs())
                    } else {
                        0
                    })
                })
                .capacity("loyalty activation counter payment exhausted");
        }
        self.pay_counters(source.id, &counter_payment);
        if let Some(kind) = definition.source_counter_cost() {
            item.source_counter_payment = Some(SourceCounterPaymentView {
                kind,
                amount: counter_payment.get(kind),
                paid: true,
            });
        }
        true
    }

    /// Record activation-use history only after all costs have been paid.
    pub(super) fn commit_activation_use(&mut self, item: &SpellView) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return;
        };
        if item.is_copy {
            return;
        }
        let Some(source) = item.source else {
            return;
        };
        let definition = source.card.activation(ability);
        let Some(object) = self.state.objects.get(source.id as usize) else {
            return;
        };
        if object.generation != source.object.generation || object.zone != definition.source_zone()
        {
            return;
        }
        let external_planeswalker_grant = item.attachment_trigger.as_ref().is_some_and(|event| {
            event.granted
                && source.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(fragment,
                        crate::cards::StaticAbilityFragment::ControlledPlaneswalkerActivation(index)
                        if *index == ability)
                    })
                })
        });
        if item.self_granted_activation.is_none() && !external_planeswalker_grant {
            if let Some(slot) = item.borrowed_slot {
                self.record_borrowed_use(source.object, slot, definition.exhaust);
            } else {
                self.record_copy_activation_use(item, ability);
            }
            self.record_power_up_use(item, ability);
        }
    }

    /// Pay the signed loyalty cost as one counter event.
    fn pay_activation_loyalty(&mut self, item: &SpellView, definition: &ActivatedAbility) {
        let source = item.source.expect("activated ability has an object source");
        let cost = definition.cost;
        if let Some(change) = cost.loyalty {
            if change < 0 {
                self.pay_counters(
                    source.id,
                    &Counters::of(CounterKind::Loyalty, u64::from(change.unsigned_abs())),
                );
            } else if change > 0 {
                self.activation_cost_counters(
                    source.object,
                    CounterKind::Loyalty,
                    change as u64,
                    item.controller,
                );
            }
        }
    }

    /// Tap is a distinct cost phase: a later counter replacement choice must
    /// not repeat the tap event when payment resumes.
    fn pay_activation_tap(
        &mut self,
        item: &mut SpellView,
        ability: u16,
        definition: &ActivatedAbility,
    ) {
        let source = item.source.expect("activated ability has an object source");
        let cost = definition.cost;
        // Capture the tap event before a subsequent sacrifice/discard/exile
        // changes the source incarnation. All other paid source costs are set.
        if cost.tap {
            if definition.mana_production().is_some() {
                let bonus = self.mountain_tap_bonus(item.controller, source.id);
                item.mana_tap_bonus = (bonus > 0).then_some(bonus);
            }
            self.tap_objects(&[source.id]);
            if let Some(grant) = &item.attachment_trigger
                && source.card.grants_activation(ability)
            {
                self.tap_objects(&[grant.grantor.card]);
            }
        }
    }

    /// A typed source-counter cost is a separate placement event from loyalty.
    fn pay_activation_source_counter(&mut self, item: &SpellView) {
        let source = item.source.expect("activated ability has an object source");
        let StackKind::ActivatedAbility {
            ability: definition_index,
        } = item.kind
        else {
            unreachable!()
        };
        if let Some((kind, amount)) = source.card.activation_put_counter(definition_index) {
            self.activation_cost_counters(source.object, kind, amount, item.controller);
        }
    }

    /// Refresh cost facts and perform departure costs after counter placements.
    fn finish_activation_source_costs(
        &mut self,
        item: &mut SpellView,
        definition: &ActivatedAbility,
    ) {
        let source = item.source.expect("activated ability has an object source");
        let cost = definition.cost;
        self.refresh_local_source_counter_quantities(item);
        // Costs and real mana activations may briefly reach the threshold before
        // the same payment sacrifices its source. Speculative planning never does.
        self.refresh_enduring_stories();
        if cost.exile_self || cost.discard || cost.sacrifice {
            // The local announcement is temporarily outside State while costs
            // commit. Freeze its post-payment source facts before departure too.
            let source = self.object_view(source.id);
            item.source
                .as_mut()
                .expect("activated ability source exists")
                .facts = DamageFacts::from_view(&source);
            item.source_keywords = source.keywords;
            item.source_tapped = source.tapped;
            item.source_attachment = source.attached_to;
        }
        if cost.exile_self {
            let event = self.move_object(source.id, Zone::Exile);
            if definition.program.iter().any(|instruction| {
                matches!(
                    instruction,
                    Instruction::GrantLandManaUntilSourceCast
                        | Instruction::ReturnCraftedSourceTransformed
                )
            }) {
                item.source_move = Some(Box::new(TriggerSourceMoveView {
                    before: event.before.reference,
                    after: event.after,
                    origin: event.before.zone,
                    destination: event.destination,
                }));
            }
        } else if cost.discard {
            self.discard_batch(&[source.id], definition.cycling.then_some(source.object));
        } else if cost.sacrifice {
            self.sacrifice_batch(&[(item.controller, source.id)]);
        }
    }
}

/// Fixed small counter costs enumerate allocations, not subsets of individual counters.
pub(super) fn counter_payments<'a>(
    cost: ActivationCost,
    available: impl Into<CounterAmounts<'a>>,
    mut emit: impl FnMut(Counters) -> bool,
) -> bool {
    let mut available = available.into();
    let reserve = cost
        .loyalty
        .filter(|change| *change < 0)
        .map_or(0, |change| u64::from(change.unsigned_abs()));
    let Some(remaining) = available.hot.loyalty.checked_sub(reserve) else {
        return false;
    };
    available.hot.loyalty = remaining;
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
