//! Resolution-owned mana choices and exact ward stack associations.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionPayment {
    pub payer: Player,
    pub cost: ManaCost,
    pub alternatives: &'static [ManaCost],
    pub life: Option<u32>,
    pub discard: bool,
    pub victim: Option<u64>,
    pub exile: bool,
    pub unpaid_pc: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct ResolutionPaymentView {
    pub payer: Player,
    pub cost: ManaCost,
    pub alternatives: Vec<ManaCost>,
    pub life: Option<u32>,
    pub victim: Option<u64>,
    pub exile: bool,
}
impl Game {
    /// Explicit counters share one check. Abilities are independent of their source spell.
    pub(super) fn counter_stack_item(&mut self, id: u64, exile: bool) -> bool {
        let Some(index) = self.state.stack.iter().position(|s| s.id == id) else {
            return false;
        };
        let victim = &self.state.stack[index];
        if victim.kind == StackKind::Spell {
            if victim.source_card.cannot_be_countered()
                || self.controlled_spells_uncounterable(victim.controller, victim.source_card)
            {
                return false;
            }
            self.move_object(
                victim.card,
                if exile { Zone::Exile } else { Zone::Graveyard },
            );
        } else {
            self.state.stack.remove(index);
        }
        true
    }

    pub(super) fn counter_permanent_exile_cast(&mut self, item: &SpellView) {
        for bound in &item.targets {
            let Some(victim) = self.bound_stack_item(bound).cloned() else {
                continue;
            };
            let permanent = PermanentType::ALL
                .into_iter()
                .any(|kind| victim.source_card.kind_mask() & kind.mask() != 0);
            if self.counter_stack_item(victim.id, permanent) && permanent && !victim.is_copy {
                let after = self.object_ref(victim.card);
                if after != victim.object
                    && self.state.objects[victim.card as usize].zone == Zone::Exile
                {
                    self.grant_exile_free_permission(item.controller, after);
                }
            }
        }
    }

    pub(super) fn target_becomes_bound(&mut self, item: &SpellView, bound: BoundTarget) {
        self.controlled_ability_target_acquired(item, bound);
        let Target::Card(id) = bound.target else {
            return;
        };
        let Some(reference) = bound.object else {
            return;
        };
        if !self.on_battlefield(id) || item.targets.iter().any(|old| old.object == Some(reference))
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
            if matched && self.queue_trigger(&source, index) && event == TriggerCondition::Ward {
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
            let has_conditional = self.state.static_sources.iter().any(|&provider| {
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
                                )
                            })
                        })
            });
            let conditional = if has_conditional {
                self.characteristic_query().conditional_ward_grants(id)
            } else {
                Vec::new()
            };
            for grant in conditional {
                let event = Box::new(AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: reference,
                    defender: None,
                    granted: true,
                    origin: self.copy_ability_origin(grant.grantor),
                });
                if self.queue_trigger_from(&source, usize::from(grant.ability), Some(event)) {
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
        if !self.state.stack.iter().any(|s| s.id == related.id) {
            return false;
        }
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
    pub(super) fn finish_ward_discard(&mut self, card: CardId) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let request = resolution.mana_payment.take().unwrap();
        assert!(request.discard);
        resolution.choice = None;
        let cause = resolution.item.controller;
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
            victim: None,
            exile: false,
            unpaid_pc,
        }));
        self.state.pending = Pending::ResolutionMana {
            player: item.controller,
        };
    }
    pub(super) fn start_resolution_life(&mut self, item: &SpellView, amount: u32) -> bool {
        let related = item.related_stack.expect("ward stack association");
        let Some(victim) = self
            .state
            .stack
            .iter()
            .find(|spell| spell.id == related.id)
            .map(|spell| StackLink {
                id: spell.id,
                controller: spell.controller,
            })
        else {
            return false;
        };
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
        self.state.players[request.payer.index()].life >= i128::from(request.life.unwrap())
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
            self.lose_life(request.payer, i128::from(request.life.unwrap()));
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
                self.lose_life(request.payer, i128::from(life));
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
