//! Resolution-owned mana choices and exact ward stack associations.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolutionPayment {
    pub payer: Player,
    pub cost: ManaCost,
    pub victim: u64,
    pub exile: bool,
    pub unpaid_pc: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct ResolutionPaymentView {
    pub payer: Player,
    pub cost: ManaCost,
    pub victim: u64,
    pub exile: bool,
}
impl Game {
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
        if source.owner == item.controller {
            return;
        }
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if ability.condition.event() == TriggerCondition::Ward
                && self.queue_trigger(&source, index)
            {
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
            victim: victim.id,
            exile,
            unpaid_pc,
        }));
        self.state.pending = Pending::ResolutionMana {
            player: victim.controller,
        };
        true
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
            for (pool, paid) in self.state.players[request.payer.index()]
                .mana
                .iter_mut()
                .zip(payment)
            {
                *pool -= paid;
            }
        } else {
            self.state.resolving.as_mut().unwrap().pc = request.unpaid_pc;
            if let Some(index) = self.state.stack.iter().position(|s| s.id == request.victim) {
                let victim = &self.state.stack[index];
                if victim.kind == StackKind::Spell {
                    self.move_object(
                        victim.card,
                        if request.exile {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        },
                    );
                } else {
                    self.state.stack.remove(index);
                }
            }
        }
        self.run_program();
    }
}
