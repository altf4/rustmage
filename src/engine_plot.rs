//! Plot is a paid special action; only its later alternative cast is a spell.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlottedCard {
    pub object: ObjectRef,
    pub owner: Player,
    pub plotted_turn: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlotPaymentView {
    pub object: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub cost: ManaCost,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlotState {
    pub cards: Vec<PlottedCard>,
    pub payment: Option<PlotPaymentView>,
}
impl PlotState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.cards.len() * std::mem::size_of::<PlottedCard>()
    }
}
impl Game {
    pub(super) fn plot_payment(&self) -> Option<&PlotPaymentView> {
        self.state.plot.as_ref().and_then(|p| p.payment.as_ref())
    }
    pub(super) fn plot_can_fund(&self, actor: Player, cost: ManaCost) -> bool {
        self.can_fund(FundingGoal {
            actor,
            required: cost.colored,
            generic: cost.generic,
            reserve: None,
            source: None,
        })
    }
    pub(super) fn begin_plot(&mut self, actor: Player, card: CardId) {
        let payment = PlotPaymentView {
            object: self.object_ref(card),
            source_card: self.state.objects[card as usize].card,
            controller: actor,
            cost: self.state.objects[card as usize].card.plot_cost().unwrap(),
        };
        self.state.plot.get_or_insert_with(Default::default).payment = Some(payment);
        self.state.pending = Pending::PlotPayment { player: actor };
        self.state.passes = 0;
    }
    pub(super) fn finish_plot(&mut self, payment: Mana) {
        let pending = self.state.plot.as_mut().unwrap().payment.take().unwrap();
        self.spend_mana(pending.controller, payment);
        let event = self.move_object(pending.object.card, Zone::Exile);
        let cards = &mut self.state.plot.as_mut().unwrap().cards;
        cards
            .try_reserve(1)
            .capacity("plotted permission allocation exhausted");
        let at = cards.partition_point(|p| p.object.card < event.after.card);
        cards.insert(
            at,
            PlottedCard {
                object: event.after,
                owner: pending.controller,
                plotted_turn: self.state.turn,
            },
        );
        self.state.pending = Pending::Priority;
        self.state.priority = pending.controller;
        self.state.passes = 0;
    }
    pub(super) fn remove_plotted(&mut self, object: ObjectRef) {
        if let Some(p) = self.state.plot.as_mut() {
            p.cards.retain(|p| p.object != object);
            if p.cards.is_empty() && p.payment.is_none() {
                self.state.plot = None;
            }
        }
    }
    pub(super) fn plotted_cast_available(&self, actor: Player, card: CardId) -> bool {
        if actor != self.state.active
            || !matches!(self.state.step, Step::Main1 | Step::Main2)
            || !self.state.stack.is_empty()
        {
            return false;
        }
        let Some(state) = &self.state.plot else {
            return false;
        };
        let Ok(i) = state.cards.binary_search_by_key(&card, |p| p.object.card) else {
            return false;
        };
        let p = state.cards[i];
        let object = &self.state.objects[card as usize];
        if object.card.is_land()
            || object.card.is_token()
            || object.card.is_emblem()
            || !object.card.is_supported()
        {
            return false;
        }
        if p.owner != actor
            || p.object != self.object_ref(card)
            || object.zone != Zone::Exile
            || object.owner != actor
            || self.state.turn <= p.plotted_turn
        {
            return false;
        }
        let plan = match object.card.effect() {
            Effect::Modal(modes) => {
                return modes
                    .iter()
                    .any(|m| self.cast_targets_available(m.targets, actor, p.object))
                    && self.spell_can_fund(
                        actor,
                        card,
                        ManaCost {
                            colored: [0; 6],
                            generic: 0,
                        },
                    );
            }
            Effect::Spell(plan, _) | Effect::Aura(plan) => plan,
            _ => TargetPlan::NONE,
        };
        self.cast_targets_available(plan, actor, p.object)
            && self.spell_can_fund(
                actor,
                card,
                ManaCost {
                    colored: [0; 6],
                    generic: 0,
                },
            )
    }
}
