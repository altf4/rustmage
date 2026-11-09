//! Optional blight declaration and ordered, replacement-aware payment.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn blight_mode_allowed(&self, item: &SpellView, mode: usize) -> bool {
        let Some(spec) = item.source.and_then(|source| source.card.blight()) else {
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
        let target_available = |bits: u64| match card.effect() {
            Effect::Modal(modes) => modes.iter().enumerate().any(|(i, mode)| {
                i < 64
                    && bits & (1u64 << i) != 0
                    && self.cast_targets_available(mode.targets, actor, self.object_ref(id))
            }),
            Effect::Spell(plan, _) => self.cast_targets_available(plan, actor, self.object_ref(id)),
            _ => false,
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
        a.initialize_cast_costs(convoke::cost_potential(
            self,
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
        ));
        let available = if let Some(modes) = a.item.modes() {
            modes.iter().enumerate().any(|(i, m)| {
                self.blight_mode_allowed(&a.item, i)
                    && self.item_targets_available(&a.item, m.targets)
            })
        } else {
            match a.item.source.expect("spell source").card.effect() {
                Effect::Spell(plan, _) => self.item_targets_available(&a.item, plan),
                _ => false,
            }
        };
        available && self.teamwork_proposal_fundable(&a)
    }
    pub(super) fn choose_blight(&mut self, pay: bool) {
        let improvise = improvise::potential(
            self,
            self.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source
                .expect("cast announcement has source")
                .card,
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn blight_x_cost(&self) -> bool {
        self.item.kind == StackKind::Spell
            && self.item.source.is_some_and(|source| {
                source
                    .card
                    .additional_cost()
                    .is_some_and(|cost| cost.kind == EffectChoiceKind::BlightXCost)
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn preview_blight_activation(
        &self,
        goal: FundingGoal,
        _spec: AdditionalCost,
    ) -> bool {
        let (source, index, counters, definition, slot) =
            goal.source.as_ref().expect("blight activation");
        let mut packet = self.activation_packet(goal.actor, source.card, *index);
        packet.set_source_definition(*definition);
        packet.borrowed_slot = *slot;
        let mut proposal = self.payment_announcement(packet, counters.clone(), None, None);
        proposal.cost = Some(ManaCost {
            colored: goal.required,
            generic: goal.generic,
        });
        proposal.total_cost = proposal.cost;
        self.activation_packet_fundable(&proposal)
    }
    pub(super) fn blight_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        // Declaration takes place before mana abilities and payments. Paying
        // counters later never changes this bound or the announced value.
        let maximum = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == a.item.controller
                    && self.is_creature(*id as CardId)
            })
            .map(|(id, _)| self.stats(id as CardId).toughness)
            .max()
            .unwrap_or(0)
            .clamp(0, i128::from(u32::MAX)) as u32;
        let fundable = |x| {
            a.cost_options()
                .any(|cost| self.blight_x_cost_fundable(a, cost, x))
        };
        match (fundable(0), maximum > 0 && fundable(1)) {
            (true, true) => vec![XInterval {
                minimum: 0,
                maximum,
            }],
            (true, false) => vec![XInterval {
                minimum: 0,
                maximum: 0,
            }],
            (false, true) => vec![XInterval {
                minimum: 1,
                maximum,
            }],
            (false, false) => Vec::new(),
        }
    }
}
