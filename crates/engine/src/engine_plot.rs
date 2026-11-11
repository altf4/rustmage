//! Plot is a paid special action; only its later alternative cast is a spell.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlottedCard {
    pub object: ObjectRef,
    pub owner: Player,
    pub plotted_turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlotPaymentView {
    pub object: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub cost: ManaCost,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlotState {
    pub cards: Vec<PlottedCard>,
    pub payment: Option<PlotPaymentView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PlotState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.cards.len() * std::mem::size_of::<PlottedCard>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn reduced_plot_cost(&self, actor: Player, cost: ManaCost) -> ManaCost {
        let reduction = self.state.static_sources.iter().copied().filter(|&id| {
            self.on_battlefield(id) && self.state.objects[id as usize].controller == actor
        }).flat_map(|id| {
            self.state.objects[id as usize].card.static_ability_groups().iter().flat_map(move |group| {
                group.fragments.iter().filter_map(move |fragment| match *fragment {
                    crate::cards::StaticAbilityFragment::PlotCostReduction(amount)
                        if self.continuous_fragment_available(id, |candidate| matches!(candidate, crate::cards::StaticAbilityFragment::PlotCostReduction(n) if *n == amount)) => Some(amount),
                    _ => None,
                })
            })
        }).fold(0u32, u32::saturating_add);
        ManaCost {
            generic: cost.generic.saturating_sub(reduction),
            ..cost
        }
    }
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
            cost: self.reduced_plot_cost(
                actor,
                self.state.objects[card as usize].card.plot_cost().unwrap(),
            ),
        };
        self.state.plot.get_or_insert_with(Default::default).payment = Some(payment);
        self.state.pending = Pending::PlotPayment { player: actor };
        self.state.passes = 0;
    }
    pub(super) fn finish_plot(&mut self, payment: Mana) {
        let pending = self.state.plot.as_mut().unwrap().payment.take().unwrap();
        self.spend_mana(pending.controller, payment);
        let event = self.move_object(pending.object.card, Zone::Exile);
        self.commit_plotted_move(&event);
        self.state.pending = Pending::Priority;
        self.state.priority = pending.controller;
        self.state.passes = 0;
    }
    pub(super) fn commit_plotted_move(&mut self, event: &ZoneChange) {
        if event.moved && event.destination == Zone::Exile && event.was_card {
            self.remember_plotted(event.before.owner, event.after);
        }
    }

    pub(super) fn remember_plotted(&mut self, owner: Player, object: ObjectRef) {
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object
            || card.zone != Zone::Exile
            || !card.is_card()
            || card.owner != owner
        {
            return;
        }
        let cards = &mut self.state.plot.get_or_insert_with(Default::default).cards;
        let plotted = PlottedCard {
            object,
            owner,
            plotted_turn: self.state.turn,
        };
        match cards.binary_search_by_key(&object.card, |entry| entry.object.card) {
            Ok(index) if cards[index].object == object => return,
            Ok(index) => cards[index] = plotted,
            Err(index) => {
                cards
                    .try_reserve(1)
                    .capacity("plotted permission allocation exhausted");
                cards.insert(index, plotted);
            }
        }
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if ability.condition.event() == TriggerCondition::SelfBecomesPlotted {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn remove_plotted(&mut self, object: ObjectRef) {
        if let Some(p) = self.state.plot.as_mut() {
            p.cards.retain(|p| p.object != object);
            if p.cards.is_empty() && p.payment.is_none() {
                self.state.plot = None;
            }
        }
    }
    pub(super) fn has_plotted_permission(&self, actor: Player, card: CardId) -> bool {
        let Some(state) = &self.state.plot else {
            return false;
        };
        let Ok(i) = state.cards.binary_search_by_key(&card, |p| p.object.card) else {
            return false;
        };
        let plotted = state.cards[i];
        let object = &self.state.objects[card as usize];
        object.is_card()
            && object.card.is_supported()
            && plotted.owner == actor
            && plotted.object == self.object_ref(card)
            && object.zone == Zone::Exile
            && object.owner == actor
            && self.state.turn > plotted.plotted_turn
    }

    pub(super) fn visit_plotted_casts(
        &self,
        actor: Player,
        card: CardId,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ]
        .into_iter()
        .any(|face| {
            self.permitted_cast_available(actor, card, CastMethod::Plot, face, true, false)
                && emit(if face == CastFace::Normal {
                    Action::AnnouncePlotted { card }
                } else {
                    Action::AnnouncePlottedFace { card, face }
                })
        })
    }
}
