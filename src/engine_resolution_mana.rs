//! Resolution-owned mana choices and exact ward stack associations.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionPayment {
    pub payer: Player,
    pub cost: ManaCost,
    pub victim: Option<u64>,
    pub exile: bool,
    pub unpaid_pc: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct ResolutionPaymentView {
    pub payer: Player,
    pub cost: ManaCost,
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
            if victim.source_card.cannot_be_countered() {
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

    pub(super) fn target_becomes_bound(&mut self, item: &SpellView, bound: BoundTarget) {
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
                let Target::Spell(card) = target.target else {
                    return None;
                };
                self.state
                    .stack
                    .iter()
                    .find(|s| {
                        s.kind == StackKind::Spell
                            && s.card == card
                            && Some(s.object) == target.object
                    })
                    .map(|s| StackLink {
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
            victim: None,
            exile: false,
            unpaid_pc,
        }));
        self.state.pending = Pending::ResolutionMana {
            player: item.controller,
        };
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
        if let Some(payment) = payment {
            self.spend_mana(request.payer, payment);
        } else {
            self.state.resolving.as_mut().unwrap().pc = request.unpaid_pc;
            if let Some(victim) = request.victim {
                self.counter_stack_item(victim, request.exile);
            }
        }
        self.run_program();
    }
}
