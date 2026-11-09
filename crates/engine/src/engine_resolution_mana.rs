//! Resolution-owned mana choices and exact ward stack associations.
use super::*;
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
        let cause = if request.discard_only {
            request.payer
        } else {
            resolution.item.controller
        };
        self.discard_batch_caused(&[card], None, Some(cause));
        self.run_program();
    }
    pub(super) fn start_resolution_payment(
        &mut self,
        item: &SpellView,
        cost: ManaCost,
        subject: CounterSubject,
        exile: bool,
        unpaid_skip: usize,
    ) -> bool {
        let victim = match subject {
            CounterSubject::Related => Some(item.related_stack.expect("ward stack association")),
            CounterSubject::Targets => item.targets.iter().find_map(|target| {
                self.bound_stack_item(target).map(|s| StackLink {
                    id: s.id,
                    controller: s.controller,
                })
            }),
        };
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
        }));
        self.state.pending = Pending::ResolutionMana {
            player: item.controller,
        };
    }
    pub(super) fn start_resolution_life(&mut self, item: &SpellView, amount: i128) -> bool {
        // Ward's "that player" is fixed by the triggering event. The optional
        // payment remains available even if another ward already countered the
        // spell, or control of that spell subsequently changed (CR118.12a).
        let victim = item.related_stack.expect("ward stack association");
        let resolution = self.state.resolving.as_mut().unwrap();
        resolution.mana_payment = Some(Box::new(ResolutionPayment {
            payer: victim.controller,
            cost: ManaCost {
                colored: [0; 6],
                generic: 0,
            },
            alternatives: &[],
            life: Some(amount),
            discard: false,
            discard_only: false,
            victim: Some(victim.id),
            exile: false,
            unpaid_pc: resolution.pc,
        }));
        self.state.pending = Pending::ResolutionLife {
            player: victim.controller,
        };
        true
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
        self.state.players[request.payer.index()].life >= request.life.unwrap()
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
        } else if let Some(victim) = request.victim {
            self.counter_stack_item(victim, false);
        }
        self.run_program();
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
            self.state.resolving.as_mut().unwrap().pc = request.unpaid_pc;
            if let Some(victim) = request.victim {
                self.counter_stack_item(victim, request.exile);
            }
        }
        self.run_program();
    }
}
