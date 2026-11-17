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

    pub(super) fn blight_intent_fundable(&self, _pay: bool) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        a.item
            .source
            .is_some_and(|source| source.card.blight().is_some())
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
    pub(super) fn blight_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        // Soul Immolation explicitly bounds X by greatest controlled creature
        // toughness at declaration. Later mana abilities or cost payments do
        // not change the chosen value, and payment feasibility does not prune it.
        let maximum = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == a.item.controller
                    && self.is_creature(*id as CardId)
            })
            .map(|(id, _)| self.stats(id as CardId).toughness)
            .max()
            .unwrap_or(0)
            .clamp(0, i128::from(u32::MAX)) as u32;
        vec![XInterval {
            minimum: 0,
            maximum,
        }]
    }
}
