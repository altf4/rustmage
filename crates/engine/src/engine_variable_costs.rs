//! Incremental ranged costs. The selected exact incarnations are reserved by
//! the same projected predicate used for fixed costs; no optional-subset search.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PaidAdditionalCostView {
    /// Actual objects sacrificed or exiled by this paid cost, after replacements.
    pub count: u64,
    /// Declared cardinality, independent of replacement-modified outcomes.
    pub selected: u64,
    /// Frozen immediately before an actual sacrifice, including redirected exile.
    pub sacrificed_vehicle: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn variable_cost_source_conflict(goal: &FundingGoal, object: ObjectRef) -> bool {
        goal.source
            .as_ref()
            .is_some_and(|(source, ability, _, definition, _)| {
                let cost = definition.activation(*ability).cost;
                *source == object && (cost.sacrifice || cost.exile_self)
            })
    }
    pub(super) fn variable_cost_group_fundable(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        chosen: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let minimum = usize::from(spec.kind.variable_cost_minimum().expect("ranged cost"));
        if chosen.iter().enumerate().any(|(i, &r)| {
            chosen[..i].contains(&r)
                || Self::variable_cost_source_conflict(&goal, r)
                || !self.additional_candidate_for_funding(spec, goal.actor, source, r)
        }) {
            return false;
        }
        if chosen.len() >= minimum {
            return self.can_fund_cost_group(goal, purpose, spec, chosen);
        }
        if exact {
            return false;
        }
        // Only fill the mandatory minimum. Larger groups are selected by the
        // player one object at a time, rather than enumerating their power set.
        (0..self.state.objects.len()).any(|id| {
            let object = self.object_ref(id as CardId);
            if chosen.contains(&object)
                || Self::variable_cost_source_conflict(&goal, object)
                || !self.additional_candidate_for_funding(spec, goal.actor, source, object)
            {
                return false;
            }
            let mut next = chosen.to_vec();
            next.push(object);
            self.variable_cost_group_fundable(goal.clone(), purpose, spec, source, &next, false)
        })
    }
    pub(super) fn variable_cost_candidate_available(
        &self,
        a: &Announcement,
        spec: AdditionalCost,
        object: ObjectRef,
    ) -> bool {
        let mut selected = a.additional_group.as_deref().cloned().unwrap_or_default();
        if a.additional_group_complete || selected.contains(&object) {
            return false;
        }
        let source = a.item.source.expect("cost source").object;
        if !self.additional_candidate_for_funding(spec, a.item.controller, source, object) {
            return false;
        }
        selected.push(object);
        if self.cast_costs_mana_paid() {
            return self.cast_cost_group_fundable(&selected, false);
        }
        self.variable_cost_group_fundable(
            self.funding_goal(a),
            self.purpose_for_item(&a.item),
            spec,
            source,
            &selected,
            false,
        )
    }
    pub(super) fn variable_cost_selection_complete(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let Some(spec) = a.additional_cost() else {
            return false;
        };
        let Some(minimum) = spec.kind.variable_cost_minimum() else {
            return false;
        };
        let selected = a.additional_group.as_deref().map_or(&[][..], Vec::as_slice);
        if a.additional_group_complete || selected.len() < usize::from(minimum) {
            return false;
        }
        if self.cast_costs_mana_paid() {
            return self.cast_cost_group_fundable(selected, true);
        }
        self.variable_cost_group_fundable(
            self.funding_goal(a),
            self.purpose_for_item(&a.item),
            spec,
            a.item.source.expect("cost source").object,
            selected,
            true,
        )
    }
    pub(super) fn finish_additional_cost_group(&mut self) {
        assert!(
            self.variable_cost_selection_complete(),
            "incomplete ranged cost"
        );
        let a = self.state.announcing.as_mut().unwrap();
        a.additional_group.get_or_insert_default();
        a.additional_group_complete = true;
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.resume_cast_cost_selection();
    }
    pub(super) fn pay_variable_additional_cost(
        &mut self,
        a: &mut Announcement,
        spec: AdditionalCost,
    ) {
        let source = a.item.source.expect("cost source").object;
        let selected = a.additional_group.as_deref().expect("selected ranged cost");
        assert!(a.additional_group_complete);
        assert!(selected.len() >= usize::from(spec.kind.variable_cost_minimum().unwrap()));
        assert!(selected.iter().enumerate().all(|(i, &r)| {
            !selected[..i].contains(&r)
                && self.additional_candidate(spec, a.item.controller, source, r)
        }));
        if self.object_ref(source.card) == source && self.on_battlefield(source.card) {
            a.item.source.as_mut().unwrap().facts =
                DamageFacts::from_view(&self.object_view(source.card));
            self.refresh_local_source_counter_quantities(&mut a.item);
        }
        let paid = match spec.kind.payment_kind() {
            EffectChoiceKind::SacrificeRange { .. } => self
                .sacrifice_cost_batch_outcome(
                    &selected
                        .iter()
                        .map(|r| (a.item.controller, r.card))
                        .collect::<Vec<_>>(),
                )
                .expect("legal ranged sacrifice cost"),
            EffectChoiceKind::ExileGraveyardRange { .. } => {
                // Capture the owned prepayment announcement for entry-redirection
                // rollback. The actual chosen group remains frozen across choices.
                let mut checkpoint = self.atomic_zone_cost_checkpoint(None);
                checkpoint.before.announcing = Some(a.clone());
                let plans = self
                    .plan_atomic_zone_cost(
                        &selected
                            .iter()
                            .map(|&r| (r, Zone::Exile))
                            .collect::<Vec<_>>(),
                    )
                    .expect("legal ranged exile cost");
                if plans
                    .iter()
                    .any(|(_, plan)| !plan.prevented && plan.destination == Zone::Battlefield)
                {
                    let paid = PaidAdditionalCostView {
                        count: plans
                            .iter()
                            .filter(|(_, plan)| !plan.prevented && plan.destination == Zone::Exile)
                            .count() as u64,
                        selected: selected.len() as u64,
                        sacrificed_vehicle: false,
                    };
                    self.stage_atomic_zone_cost(plans, checkpoint);
                    paid
                } else {
                    let events = self
                        .commit_atomic_zone_cost(plans)
                        .expect("valid ranged exile batch");
                    let paid = PaidAdditionalCostView {
                        count: events
                            .iter()
                            .filter(|event| event.moved && event.destination == Zone::Exile)
                            .count() as u64,
                        selected: selected.len() as u64,
                        sacrificed_vehicle: false,
                    };
                    self.finish_zone_events(&events, &[]);
                    paid
                }
            }
            _ => unreachable!("ranged cost"),
        };
        a.item.paid_additional_cost = Some(Box::new(paid));
        a.additional_paid = true;
    }
}
