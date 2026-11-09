//! Resolution-owned payments and exact stack associations.
use super::*;
use crate::cards::{ResolutionCost, ResolutionObjectCost};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionPayment {
    pub payer: Player,
    pub cost: ManaCost,
    pub alternatives: &'static [ManaCost],
    pub life: Option<i128>,
    pub discard: bool,
    pub discard_only: bool,
    pub victim: Option<u64>,
    pub exile: bool,
    pub unpaid_pc: usize,
    pub choices: Option<ResolutionCostChoices>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionCostChoices {
    pub options: Vec<ResolutionCostView>,
    pub selected: Option<u16>,
    pub objects: Vec<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ResolutionCostView {
    pub mana: ManaCost,
    pub life: Option<i128>,
    pub objects: Option<ResolutionObjectCost>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ResolutionPayment {
    fn object_cost(&self) -> Option<ResolutionObjectCost> {
        let choices = self.choices.as_ref()?;
        choices.options[usize::from(choices.selected?)].objects
    }

    pub(super) fn route_selected(&self) -> bool {
        self.choices
            .as_ref()
            .is_none_or(|choices| choices.selected.is_some())
    }

    pub(super) fn view(&self, private: bool) -> ResolutionPaymentView {
        ResolutionPaymentView {
            payer: self.payer,
            cost: self.cost,
            alternatives: self.alternatives.to_vec(),
            life: self.life,
            discard_only: self.discard_only,
            victim: self.victim,
            exile: self.exile,
            costs: self
                .choices
                .as_ref()
                .map_or_else(Vec::new, |c| c.options.clone()),
            selected_cost: self.choices.as_ref().and_then(|c| c.selected),
            selected_objects: self
                .choices
                .as_ref()
                .filter(|_| private)
                .map_or_else(Vec::new, |c| c.objects.clone()),
        }
    }

    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.choices.as_ref().map_or(0, |choices| {
                std::mem::size_of_val(choices.options.as_slice())
                    + std::mem::size_of_val(choices.objects.as_slice())
            })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct ResolutionPaymentView {
    pub payer: Player,
    pub cost: ManaCost,
    pub alternatives: Vec<ManaCost>,
    pub life: Option<i128>,
    #[serde(skip_serializing_if = "is_false")]
    pub discard_only: bool,
    pub victim: Option<u64>,
    pub exile: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub costs: Vec<ResolutionCostView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_cost: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub selected_objects: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !*value
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn stack_item_is_counterable(&self, victim: &SpellView) -> bool {
        victim.kind != StackKind::Spell
            || victim.source.is_some_and(|source| {
                !self
                    .state
                    .uncounterable_spell_ids
                    .as_deref()
                    .is_some_and(|ids| ids.contains(&victim.id))
                    && !source.card.cannot_be_countered()
                    && !self.controlled_spells_uncounterable(victim.controller, source.card)
            })
    }
    /// One simultaneous counter event; copies and abilities each count once.
    pub(super) fn counter_opponents_counted(&mut self, controller: Player) {
        let victims: Vec<_> = self
            .state
            .stack
            .iter()
            .filter(|victim| {
                victim.controller != controller && self.stack_item_is_counterable(victim)
            })
            .cloned()
            .collect();
        let spells: Vec<_> = victims
            .iter()
            .filter(|victim| victim.kind == StackKind::Spell)
            .filter_map(|victim| victim.source.map(|source| source.id))
            .collect();
        for victim in victims
            .iter()
            .filter(|victim| victim.kind != StackKind::Spell)
        {
            self.retain_departing_activation_for_triggers(victim);
        }
        self.state.stack.retain(|item| {
            !victims
                .iter()
                .any(|victim| victim.id == item.id && victim.kind != StackKind::Spell)
        });
        let events = self.move_objects_raw(&spells, Zone::Graveyard);
        self.state.resolving.as_mut().unwrap().moved_count = Some(
            victims
                .len()
                .try_into()
                .capacity("countered item count exhausted"),
        );
        self.finish_zone_events(&events, &[]);
    }
    /// Explicit counters share one check. Abilities are independent of their source spell.
    pub(super) fn counter_stack_item(&mut self, id: u64, exile: bool) -> bool {
        let Some(index) = self.state.stack.iter().position(|s| s.id == id) else {
            return false;
        };
        let victim = &self.state.stack[index];
        if !self.stack_item_is_counterable(victim) {
            return false;
        }
        if victim.kind == StackKind::Spell {
            let Some(source) = victim.source else {
                return false;
            };
            self.move_object(source.id, if exile { Zone::Exile } else { Zone::Graveyard });
        } else {
            let departed = self.state.stack.remove(index);
            self.retain_departing_activation_for_triggers(&departed);
        }
        true
    }

    pub(super) fn counter_permanent_exile_cast(&mut self, item: &SpellView) {
        for bound in &item.targets {
            let Some(victim) = self.bound_stack_item(bound).cloned() else {
                continue;
            };
            let Some(source) = victim.source else {
                continue;
            };
            let permanent = victim.kind == StackKind::Spell
                && PermanentType::ALL
                    .into_iter()
                    .any(|kind| source.card.kind_mask() & kind.mask() != 0);
            if self.counter_stack_item(victim.id, permanent) && permanent && !victim.is_copy {
                let after = self.object_ref(source.id);
                if after != source.object
                    && self.state.objects[source.id as usize].zone == Zone::Exile
                {
                    self.grant_exile_free_permission(item.controller, after);
                }
            }
        }
    }

    pub(super) fn target_becomes_bound(&mut self, item: &SpellView, bound: BoundTarget) {
        self.controlled_ability_target_acquired(item, bound);
        self.opponent_targets_controlled_creature(item, bound);
        let Target::Card(id) = bound.target else {
            return;
        };
        let Some(reference) = bound.object else {
            return;
        };
        if !self.on_battlefield(id)
            || self.object_ref(id) != reference
            || item.targets.iter().any(|old| old.object == Some(reference))
        {
            return;
        }
        let source = self.object_view(id);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            let event = ability.condition.event();
            let matched = match event {
                TriggerCondition::Ward => source.controller != item.controller,
                TriggerCondition::SelfTargeted {
                    relation,
                    spells_only,
                } => {
                    (!spells_only || item.kind == StackKind::Spell)
                        && match relation {
                            CastRelation::You => source.controller == item.controller,
                            CastRelation::Opponent => source.controller != item.controller,
                            CastRelation::Any => true,
                        }
                }
                _ => false,
            };
            if matched && self.queue_trigger(&source, index) {
                self.state
                    .pending_triggers
                    .last_mut()
                    .unwrap()
                    .refer_sources(item.semantic_stack_reference());
                if event != TriggerCondition::Ward {
                    continue;
                }
                self.state
                    .pending_triggers
                    .last_mut()
                    .unwrap()
                    .related_stack = Some(StackLink {
                    id: item.id,
                    controller: item.controller,
                });
            }
        }
        if source.controller != item.controller {
            // The active layer-one definition is materialized for copied providers.
            // Keep blank providers in this conservative gate: restored abilities
            // must work without reacquiring or rebuilding a derived ward cache.
            let has_static_ward = self.state.static_sources.iter().any(|&provider| {
                self.on_battlefield(provider)
                    && self.state.objects[provider as usize]
                        .card
                        .static_ability_groups()
                        .iter()
                        .any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    crate::cards::StaticAbilityFragment::ConditionalWard { .. }
                                        | crate::cards::StaticAbilityFragment::ControlledCreatureTrigger { .. }
                                )
                            })
                        })
            });
            let static_grants = if has_static_ward {
                self.characteristic_query().static_ward_grants(id)
            } else {
                Vec::new()
            };
            for grant in static_grants {
                let event = Box::new(AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: reference,
                    defender: None,
                    granted: true,
                    origin: self.copied_grant_origin(grant.grantor),
                });
                if self.queue_trigger_from(
                    &source,
                    usize::from(grant.ability.expect("static ward has a trigger template")),
                    Some(event),
                ) {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_sources(item.semantic_stack_reference());
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .related_stack = Some(StackLink {
                        id: item.id,
                        controller: item.controller,
                    });
                }
            }
            let grants: Vec<_> = self
                .state
                .attachment_sources
                .iter()
                .filter(|&&id| self.attachment_recipient(id) == Some(reference))
                .map(|&id| self.object_view(id))
                .collect();
            for giver in grants {
                for (index, ability) in giver.card.triggers().iter().enumerate() {
                    if ability.condition.is_granted()
                        && ability.condition.event() == TriggerCondition::Ward
                    {
                        let grant = Box::new(AttachmentTriggerView {
                            grantor: giver.reference,
                            granting_card: giver.card,
                            recipient: reference,
                            defender: None,
                            granted: true,
                            origin: giver.copied_grant_origin(),
                        });
                        if self.queue_trigger_from(&source, index, Some(grant)) {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .refer_sources(item.semantic_stack_reference());
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .related_stack = Some(StackLink {
                                id: item.id,
                                controller: item.controller,
                            });
                        }
                    }
                }
            }
        }
    }
    pub(super) fn start_ward_discard_or_mana(&mut self, item: &SpellView, cost: ManaCost) -> bool {
        let related = item.related_stack.expect("ward stack association");
        self.start_resolution_payment(item, cost, CounterSubject::Related, false, 0);
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .as_mut()
            .unwrap()
            .discard = true;
        self.start_ward_discard_choice(item, related.controller);
        true
    }
    pub(super) fn start_ward_discard_only(&mut self, item: &SpellView) -> bool {
        if !self.start_ward_discard_or_mana(
            item,
            ManaCost {
                colored: [0; 6],
                generic: 0,
            },
        ) {
            return false;
        }
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .as_mut()
            .unwrap()
            .discard_only = true;
        true
    }
    pub(super) fn finish_ward_discard(&mut self, card: CardId) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let request = resolution.mana_payment.take().unwrap();
        assert!(request.discard);
        resolution.choice = None;
        let cause = resolution.item.controller;
        self.discard_batch_caused(&[card], None, Some(cause));
        self.run_program();
    }
    fn resolution_payment_subject(
        &self,
        item: &SpellView,
        subject: CounterSubject,
    ) -> Option<StackLink> {
        match subject {
            // The triggering payer is fixed even if the stack item leaves or changes
            // controller before this payment is offered (CR 118.12a).
            CounterSubject::Related => Some(item.related_stack.expect("ward stack association")),
            CounterSubject::Targets => item.targets.iter().find_map(|target| {
                self.bound_stack_item(target).map(|s| StackLink {
                    id: s.id,
                    controller: s.controller,
                })
            }),
        }
    }

    pub(super) fn start_resolution_payment(
        &mut self,
        item: &SpellView,
        cost: ManaCost,
        subject: CounterSubject,
        exile: bool,
        unpaid_skip: usize,
    ) -> bool {
        let victim = self.resolution_payment_subject(item, subject);
        let resolution = self.state.resolving.as_mut().unwrap();
        let unpaid_pc = resolution
            .pc
            .checked_add(unpaid_skip)
            .expect("program offset");
        assert!(unpaid_pc <= resolution.program.len());
        let Some(victim) = victim else {
            resolution.pc = unpaid_pc;
            return false;
        };
        resolution.mana_payment = Some(Box::new(ResolutionPayment {
            payer: victim.controller,
            cost,
            alternatives: &[],
            life: None,
            discard: false,
            discard_only: false,
            victim: Some(victim.id),
            exile,
            unpaid_pc,
            choices: None,
        }));
        self.state.pending = Pending::ResolutionMana {
            player: victim.controller,
        };
        true
    }
    pub(super) fn start_optional_resolution_payment(
        &mut self,
        item: &SpellView,
        cost: ManaCost,
        skip: usize,
    ) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let unpaid_pc = resolution.pc.checked_add(skip).expect("program offset");
        assert!(unpaid_pc <= resolution.program.len());
        resolution.mana_payment = Some(Box::new(ResolutionPayment {
            payer: item.controller,
            cost,
            alternatives: &[],
            life: None,
            discard: false,
            discard_only: false,
            victim: None,
            exile: false,
            unpaid_pc,
            choices: None,
        }));
        self.state.pending = Pending::ResolutionMana {
            player: item.controller,
        };
    }
    pub(super) fn start_resolution_life(&mut self, item: &SpellView, amount: i128) -> bool {
        if !self.start_resolution_payment(item, ManaCost::ZERO, CounterSubject::Related, false, 0) {
            return false;
        }
        let request = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .as_mut()
            .unwrap();
        request.life = Some(amount);
        self.state.pending = Pending::ResolutionLife {
            player: request.payer,
        };
        true
    }

    pub(super) fn start_resolution_costs(
        &mut self,
        item: &SpellView,
        subject: Option<CounterSubject>,
        costs: &'static [ResolutionCost],
        exile: bool,
        unpaid_skip: usize,
    ) -> bool {
        assert!(!costs.is_empty() && costs.len() <= usize::from(u16::MAX));
        if let Some(subject) = subject {
            if !self.start_resolution_payment(item, ManaCost::ZERO, subject, exile, unpaid_skip) {
                return false;
            }
        } else {
            self.start_optional_resolution_payment(item, ManaCost::ZERO, unpaid_skip);
        }
        let options = costs
            .iter()
            .map(|cost| {
                let extra = cost.extra_generic.map_or(0, |amount| {
                    u32::try_from(self.item_amount(amount, item))
                        .capacity("resolution mana cost exhausted")
                });
                ResolutionCostView {
                    mana: ManaCost {
                        generic: cost
                            .mana
                            .generic
                            .checked_add(extra)
                            .capacity("resolution mana cost exhausted"),
                        ..cost.mana
                    },
                    life: cost.life.map(|amount| {
                        i128::try_from(self.item_amount(amount, item))
                            .capacity("life payment exhausted")
                    }),
                    objects: cost.objects.inspect(|requirement| {
                        assert!(
                            matches!(
                                requirement.cost.kind.payment_kind(),
                                EffectChoiceKind::Sacrifice
                                    | EffectChoiceKind::SacrificePermanents(_)
                                    | EffectChoiceKind::Discard
                                    | EffectChoiceKind::DiscardNamed(_)
                                    | EffectChoiceKind::DiscardMatchingLegendary
                                    | EffectChoiceKind::DiscardLegendary
                                    | EffectChoiceKind::ExileGraveyard
                                    | EffectChoiceKind::ExileGraveyardCards(_)
                                    | EffectChoiceKind::ExileControlled
                                    | EffectChoiceKind::ExileControlledOrGraveyard
                                    | EffectChoiceKind::TapPermanents(_)
                                    | EffectChoiceKind::ReturnToHand
                            ),
                            "unsupported fixed object payment"
                        );
                    }),
                }
            })
            .collect();
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .as_mut()
            .unwrap()
            .choices = Some(ResolutionCostChoices {
            options,
            selected: None,
            objects: Vec::new(),
        });
        if costs.len() == 1 {
            self.choose_resolution_cost(0);
        }
        true
    }

    pub(super) fn choose_resolution_cost(&mut self, index: u16) {
        let request = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .as_mut()
            .unwrap();
        let choices = request.choices.as_mut().unwrap();
        assert!(choices.selected.is_none());
        let choice = choices.options[usize::from(index)];
        choices.selected = Some(index);
        request.cost = choice.mana;
        request.life = choice.life;
        if choice.mana == ManaCost::ZERO && choice.life.is_some() && choice.objects.is_none() {
            self.state.pending = Pending::ResolutionLife {
                player: request.payer,
            };
        }
    }

    fn resolution_cost_candidate(
        &self,
        payer: Player,
        cost: AdditionalCost,
        object: ObjectRef,
    ) -> bool {
        self.state
            .resolving
            .as_ref()
            .unwrap()
            .item
            .source
            .is_some_and(|source| self.additional_candidate(cost, payer, source.object, object))
    }

    pub(super) fn visit_resolution_cost_selections(
        &self,
        request: &ResolutionPayment,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(choices) = &request.choices else {
            return false;
        };
        if choices.selected.is_none() {
            return (0..choices.options.len()).any(|index| {
                emit(Action::ChooseResolutionCost {
                    index: index as u16,
                })
            });
        }
        let Some(required) = request.object_cost() else {
            return false;
        };
        if self.resolution_objects_ready(request) {
            return false;
        }
        let candidates: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, _)| self.object_ref(id as CardId))
            .filter(|&object| self.resolution_cost_candidate(request.payer, required.cost, object))
            .collect();
        candidates.len() >= usize::from(required.count)
            && candidates.into_iter().any(|object| {
                !choices.objects.contains(&object) && emit(Action::ChooseAdditionalCost { object })
            })
    }

    pub(super) fn resolution_objects_ready(&self, request: &ResolutionPayment) -> bool {
        request.object_cost().is_none_or(|required| {
            let objects = &request.choices.as_ref().unwrap().objects;
            objects.len() == usize::from(required.count)
                && objects.iter().all(|&object| {
                    self.resolution_cost_candidate(request.payer, required.cost, object)
                })
        })
    }

    pub(super) fn choose_resolution_cost_object(&mut self, object: ObjectRef) {
        let request = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .take()
            .unwrap();
        let required = request.object_cost().unwrap();
        let mut request = request;
        let objects = &mut request.choices.as_mut().unwrap().objects;
        objects.retain(|&selected| {
            self.resolution_cost_candidate(request.payer, required.cost, selected)
        });
        assert!(objects.len() < usize::from(required.count) && !objects.contains(&object));
        objects.push(object);
        self.state.resolving.as_mut().unwrap().mana_payment = Some(request);
    }

    fn pay_resolution_objects(
        &mut self,
        payer: Player,
        required: AdditionalCost,
        objects: &[ObjectRef],
    ) {
        let cards: Vec<_> = objects.iter().map(|object| object.card).collect();
        match required.kind.payment_kind() {
            EffectChoiceKind::Sacrifice | EffectChoiceKind::SacrificePermanents(_) => {
                let sacrifices: Vec<_> = cards.into_iter().map(|card| (payer, card)).collect();
                assert!(
                    self.sacrifice_cost_batch_atomic(&sacrifices),
                    "legal sacrifice cost"
                );
            }
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => {
                let cause = self.state.resolving.as_ref().unwrap().item.controller;
                self.discard_batch_caused(&cards, None, Some(cause));
            }
            EffectChoiceKind::ExileGraveyard
            | EffectChoiceKind::ExileGraveyardCards(_)
            | EffectChoiceKind::ExileControlled
            | EffectChoiceKind::ExileControlledOrGraveyard => {
                let events = self.move_objects_raw(&cards, Zone::Exile);
                self.finish_zone_events(&events, &[]);
            }
            EffectChoiceKind::TapPermanents(_) => self.tap_objects(&cards),
            EffectChoiceKind::ReturnToHand => {
                let events = self.move_objects_raw(&cards, Zone::Hand);
                self.finish_zone_events(&events, &[]);
            }
            _ => unreachable!("unsupported resolution object cost"),
        }
    }
    pub(super) fn can_pay_resolution_life(&self) -> bool {
        let request = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .mana_payment
            .as_ref()
            .unwrap();
        let life = request.life.unwrap();
        life == 0 || self.state.players[request.payer.index()].life >= life
    }
    pub(super) fn finish_resolution_life(&mut self, pay: bool) {
        let request = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .take()
            .unwrap();
        if pay {
            self.pay_life(request.payer, request.life.unwrap());
        } else {
            self.decline_resolution_payment(&request);
        }
        self.run_program();
    }
    fn decline_resolution_payment(&mut self, request: &ResolutionPayment) {
        self.state.resolving.as_mut().unwrap().pc = request.unpaid_pc;
        if let Some(victim) = request.victim {
            self.counter_stack_item(victim, request.exile);
        }
    }
    pub(super) fn finish_resolution_payment(&mut self, payment: Option<Mana>) {
        let request = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .mana_payment
            .take()
            .unwrap();
        if request.discard {
            self.state.resolving.as_mut().unwrap().choice = None;
        }
        if let Some(payment) = payment {
            self.spend_mana(request.payer, payment);
            if let Some(life) = request.life {
                self.pay_life(request.payer, life);
            }
            if let Some(required) = request.object_cost() {
                let objects = &request.choices.as_ref().unwrap().objects;
                self.pay_resolution_objects(request.payer, required.cost, objects);
            }
            let r = self.state.resolving.as_ref().unwrap();
            if let Some(
                Instruction::PayXReflexive { ability }
                | Instruction::PayXReflexiveLimited { ability, .. },
            ) = r.program.get(r.pc.saturating_sub(1))
            {
                let (item, ability) = (r.item.clone(), *ability);
                self.queue_reflexive(&item, ability, None);
            }
        } else {
            self.decline_resolution_payment(&request);
        }
        self.run_program();
    }
}
