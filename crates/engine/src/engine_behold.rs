//! Declared, ordered behold costs share the ordinary casting payment context.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct BeholdCostView {
    pub subtype: &'static str,
    pub candidates: Vec<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn needs_behold_type(&self) -> bool {
        self.item.kind == StackKind::Spell
            && self.item.cast.is_some_and(|c| !c.behold_type_declared)
            && self
                .item
                .source
                .and_then(|s| s.card.additional_cost())
                .is_some_and(|s| matches!(s.kind, EffectChoiceKind::OptionalBeholdType(_)))
    }
    pub(super) fn needs_behold(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("cast source")
                .card
                .behold_cast()
                .is_some()
            && self.item.cast.is_some_and(|cast| cast.beheld.is_none())
    }
    pub(super) fn behold_decline_generic(&self) -> u32 {
        self.item
            .cast
            .filter(|cast| cast.beheld == Some(false))
            .and_then(|_| self.item.source?.card.behold_cast())
            .map_or(0, |spec| spec.decline_generic)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn behold_type_fundable(&self, kind: Option<crate::cards::CreatureType>) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        let cast = a.item.cast.as_mut().unwrap();
        cast.beheld_type = kind;
        cast.behold_type_declared = true;
        a.cost_options()
            .any(|cost| self.behold_announcement_fundable(&a, cost))
    }
    pub(super) fn choose_behold_type(&mut self, kind: Option<crate::cards::CreatureType>) {
        let cast = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .item
            .cast
            .as_mut()
            .unwrap();
        cast.beheld_type = kind;
        cast.behold_type_declared = true;
        self.continue_announcement_choices(false);
    }
    pub(super) fn behold_announcement_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
        if a.item.mode.is_none()
            && let Some(modes) = a.item.modes()
        {
            return modes.iter().enumerate().any(|(index, mode)| {
                let mut proposal = a.clone();
                proposal.item.mode = Some(u16::try_from(index).expect("mode index"));
                proposal.plan = mode.targets;
                self.behold_announcement_fundable(&proposal, cost)
            });
        }
        if a.needs_bargain() {
            return [false, true].into_iter().any(|pay| {
                let mut proposal = a.clone();
                proposal.item.cast.as_mut().unwrap().bargain = Some(pay);
                self.behold_announcement_fundable(&proposal, cost)
            });
        }
        let mut proposal = a.clone();
        proposal.cost = Some(cost);
        proposal.cast_costs.get_or_insert_with(Default::default);
        proposal.initialize_cast_costs(convoke::cost_potential(
            self,
            proposal.item.source.unwrap().card,
        ));
        let plan = if proposal.plan == TargetPlan::NONE {
            self.target_spec(&proposal.item)
        } else {
            proposal.plan
        };
        let Some(total) = self.target_proposal_total(
            &proposal,
            Self::announcement_base_cost(&proposal),
            plan,
            None,
            false,
            Self::announcement_optional_reduction(&proposal),
        ) else {
            return false;
        };
        // This shared proposal funder installs the same complete cost context;
        // no Harmonize reduction is applied when its selected object is absent.
        self.harmonize_proposal_fundable(&proposal, None, total)
    }
    pub(super) fn behold_timing_required(&self, item: &SpellView) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        item.kind == StackKind::Spell
            && source
                .card
                .behold_cast()
                .is_some_and(|spec| spec.grants_flash)
            && self.state.resolving.is_none()
            && !item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Sneak || cast.flash_surcharge)
            && !source.card.is_instant()
            && !source.card.info().keywords.contains(Keywords::FLASH)
            && !self.cast_timing_granted(item.controller, source.card)
            && !(self.state.active == item.controller
                && matches!(self.state.step, Step::Main1 | Step::Main2)
                && self.state.stack.is_empty())
    }
    pub(super) fn behold_intent_fundable(&self, a: &Announcement, pay: bool) -> bool {
        if !pay && a.behold_required {
            return false;
        }
        let mut proposal = a.clone();
        proposal.item.cast.as_mut().expect("cast receipt").beheld = Some(pay);
        // Intent precedes the mana equivalent; preview all permitted equivalents.
        proposal.cost = None;
        proposal.initialize_cast_costs(convoke::cost_potential(
            self,
            proposal.item.source.unwrap().card,
        ));
        proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }
    pub(super) fn choose_cast_behold(&mut self, pay: bool) {
        self.state
            .announcing
            .as_mut()
            .unwrap()
            .item
            .cast
            .as_mut()
            .unwrap()
            .beheld = Some(pay);
        self.continue_announcement_choices(false);
    }
    fn behold_cost_subtype(&self) -> &'static str {
        let a = self.state.announcing.as_ref().unwrap();
        let costs = a.cast_costs.as_ref().unwrap();
        match costs.clauses[usize::from(costs.active.unwrap())].clause {
            CastCostClause::Behold(subtype) => subtype,
            _ => unreachable!("active behold cost"),
        }
    }
    pub(super) fn behold_cost_candidates(&self) -> Vec<ObjectRef> {
        let a = self.state.announcing.as_ref().unwrap();
        hand_choices::behold_candidates(self, a.item.controller, self.behold_cost_subtype(), &[])
    }
    pub(super) fn behold_cost_view(&self, viewer: Player) -> Option<BeholdCostView> {
        let Pending::BeholdCost { player } = self.state.pending else {
            return None;
        };
        self.shares_information(viewer, player)
            .then(|| BeholdCostView {
                subtype: self.behold_cost_subtype(),
                candidates: self.behold_cost_candidates(),
            })
    }
    pub(super) fn pay_behold_cost(&mut self, object: ObjectRef) {
        if self.state.objects[object.card as usize].zone == Zone::Hand {
            self.remember_public_hand(object);
        }
        let a = self.state.announcing.as_mut().unwrap();
        let costs = a.cast_costs.as_mut().unwrap();
        let index = usize::from(costs.active.take().unwrap());
        costs.clauses[index].paid = true;
        self.finish_announcement();
    }
}
