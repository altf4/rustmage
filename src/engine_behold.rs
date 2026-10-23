//! Additional casting declarations and non-consuming exact Behold witnesses.
use super::*;
use crate::cards::CreatureType;
impl Announcement {
    pub(super) fn needs_behold_type(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source_card
                .additional_cost()
                .is_some_and(|spec| matches!(spec.kind, EffectChoiceKind::OptionalBeholdType(_)))
            && self
                .item
                .cast
                .is_some_and(|cast| cast.optional_additional.is_none())
    }
}
impl Game {
    pub(super) fn choose_behold_type(&mut self, creature_type: Option<CreatureType>) {
        let a = self.state.announcing.as_mut().unwrap();
        a.item.source_chosen_type = creature_type;
        a.item.cast.as_mut().unwrap().optional_additional = Some(creature_type.is_some());
        let needs_mode = a.item.modes().is_some() && a.item.mode.is_none();
        self.continue_announcement_choices(needs_mode);
    }
    pub(super) fn behold_type_choice_fundable(
        &self,
        a: &Announcement,
        creature_type: Option<CreatureType>,
    ) -> bool {
        let mut proposal = a.clone();
        proposal.item.source_chosen_type = creature_type;
        proposal.item.cast.as_mut().unwrap().optional_additional = Some(creature_type.is_some());
        self.behold_group_fundable(&proposal, None, &[])
    }
    pub(super) fn behold_group_fundable(
        &self,
        a: &Announcement,
        cost: Option<ManaCost>,
        selected: &[ObjectRef],
    ) -> bool {
        let Some(spec) = a.additional_cost() else {
            let mut proposal = a.clone();
            if let Some(cost) = cost {
                proposal.cost = Some(cost);
                proposal.total_cost = None;
            }
            return self.can_fund_for(
                self.funding_goal(&proposal),
                self.purpose_for_item(&proposal.item),
            );
        };
        let EffectChoiceKind::BeholdMany(amount) = spec.kind else {
            return false;
        };
        if selected.len() > usize::from(amount)
            || selected.iter().enumerate().any(|(index, &reference)| {
                selected[..index].contains(&reference)
                    || !self.additional_candidate(spec, a.item.controller, a.item.object, reference)
            })
        {
            return false;
        }
        if self.linked_funding_can_probe(a.item.controller) {
            let mut proposal = a.clone();
            if let Some(cost) = cost {
                proposal.cost = Some(cost);
                proposal.total_cost = None;
            }
            if let Some(result) = self.linked_deferred_funding(
                self.funding_goal(&proposal),
                self.purpose_for_item(&a.item),
                linked_funding::DeferredFundingCost::BeholdGroup {
                    announcement: Box::new(proposal),
                    cost,
                    selected: selected.to_vec(),
                },
            ) {
                return result;
            }
        }
        if selected.len() == usize::from(amount) {
            return self.behold_witnesses_fundable(a, spec, selected, cost);
        }
        let candidates: Vec<_> = (0..self.state.objects.len())
            .map(|id| self.object_ref(id as CardId))
            .filter(|reference| {
                !selected.contains(reference)
                    && self.additional_candidate(spec, a.item.controller, a.item.object, *reference)
            })
            .collect();
        fn complete(
            game: &Game,
            a: &Announcement,
            spec: AdditionalCost,
            cost: Option<ManaCost>,
            amount: usize,
            selected: &mut Vec<ObjectRef>,
            candidates: &[ObjectRef],
        ) -> bool {
            if selected.len() == amount {
                return game.behold_witnesses_fundable(a, spec, selected, cost);
            }
            if selected.len() + candidates.len() < amount {
                return false;
            }
            for (index, &reference) in candidates.iter().enumerate() {
                selected.push(reference);
                if complete(
                    game,
                    a,
                    spec,
                    cost,
                    amount,
                    selected,
                    &candidates[index + 1..],
                ) {
                    return true;
                }
                selected.pop();
            }
            false
        }
        complete(
            self,
            a,
            spec,
            cost,
            usize::from(amount),
            &mut selected.to_vec(),
            &candidates,
        )
    }
}
