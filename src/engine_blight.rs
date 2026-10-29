//! Optional blight declaration and ordered, replacement-aware payment.
use super::*;

impl Game {
    pub(super) fn blight_mode_allowed(&self, item: &SpellView, mode: usize) -> bool {
        let Some(spec) = item.source_card.blight() else {
            return true;
        };
        let bits = if item.cast.is_some_and(|c| c.blight == Some(true)) {
            spec.paid_modes
        } else {
            spec.normal_modes
        };
        mode < 64 && bits & (1u64 << mode) != 0
    }
    pub(super) fn blight_initial_fundable(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> bool {
        let card = self.state.objects[id as usize].card;
        let spec = card.blight().unwrap();
        let Effect::Modal(modes) = card.effect() else {
            return false;
        };
        let target_available = |bits: u64| {
            modes.iter().enumerate().any(|(i, mode)| {
                i < 64
                    && bits & (1u64 << i) != 0
                    && self.cast_targets_available(mode.targets, actor, self.object_ref(id))
            })
        };
        if target_available(spec.normal_modes) && self.spell_can_fund(actor, id, base) {
            return true;
        }
        if !target_available(spec.paid_modes) {
            return false;
        }
        let Some(cost) = self.reduced_spell_cost(actor, id, base) else {
            return false;
        };
        self.preview_optional_cast_costs(
            FundingGoal {
                actor,
                required: cost.colored,
                generic: cost.generic,
                source: None,
                reserve: None,
            },
            self.spell_mana_purpose_for_card(actor, id, card),
            self.object_ref(id),
            card.additional_cost(),
            None,
            &[],
            false,
            &[],
            &[CastCostClause::Blight {
                amount: spec.amount,
                selected: None,
            }],
        )
        .unwrap_or(false)
    }
    pub(super) fn blight_intent_fundable(&self, pay: bool) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        a.item.cast.as_mut().unwrap().blight = Some(pay);
        a.initialize_cast_costs(improvise::potential(self, a.item.source_card));
        let available = a.item.modes().is_some_and(|modes| {
            modes.iter().enumerate().any(|(i, m)| {
                self.blight_mode_allowed(&a.item, i)
                    && self.item_targets_available(&a.item, m.targets)
            })
        });
        available && self.teamwork_proposal_fundable(&a)
    }
    pub(super) fn choose_blight(&mut self, pay: bool) {
        let improvise = improvise::potential(
            self,
            self.state.announcing.as_ref().unwrap().item.source_card,
        );
        let a = self.state.announcing.as_mut().unwrap();
        a.item.cast.as_mut().unwrap().blight = Some(pay);
        a.initialize_cast_costs(improvise);
        self.continue_announcement_choices(false);
    }
    pub(super) fn visit_blight_payments(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if self.blight_subject_fundable(object) && emit(Action::PayBlight { object }) {
                return true;
            }
        }
        false
    }
    pub(super) fn pay_blight(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        let cost = a.cast_costs.as_mut().unwrap();
        let active = cost.active.take().unwrap();
        let CastCostClause::Blight { amount, .. } = cost.clauses[usize::from(active)].clause else {
            unreachable!()
        };
        cost.clauses[usize::from(active)].clause = CastCostClause::Blight {
            amount,
            selected: Some(object),
        };
        cost.clauses[usize::from(active)].paid = true;
        let player = a.item.controller;
        self.state.pending = Pending::CastCosts { player };
        self.blight_cost_counters(object, amount, player);
    }
}
