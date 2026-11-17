//! Optional teamwork declarations and simultaneous, non-targeting creature-tap costs.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TeamworkChoiceView {
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn teamwork_mode_allowed(&self, item: &SpellView, mode: usize) -> bool {
        let Some(spec) = item.source.and_then(|source| source.card.teamwork()) else {
            return true;
        };
        let bits = if item.cast.is_some_and(|c| c.teamwork == Some(true)) {
            spec.paid_modes
        } else {
            spec.normal_modes
        };
        mode < 64 && bits & (1u64 << mode) != 0
    }
    pub(super) fn teamwork_intent_fundable(&self, announcement: &Announcement, _pay: bool) -> bool {
        // This declares an optional cost before the mana window. The supplied
        // teammates' total power is checked when that cost is paid.
        announcement
            .item
            .source
            .is_some_and(|source| source.card.teamwork().is_some())
    }
    pub(super) fn choose_teamwork(&mut self, pay: bool) {
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
        a.item.cast.as_mut().unwrap().teamwork = Some(pay);
        a.initialize_cast_costs(improvise);
        let item = a.item.clone();
        let plan = self.target_spec(&item);
        self.state.announcing.as_mut().unwrap().plan = plan;
        self.continue_announcement_choices(false);
    }
    pub(super) fn visit_teamwork_payments(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let selected = &a.cast_costs.as_ref().unwrap().teammates;
        for (id, o) in self.state.objects.iter().enumerate() {
            let object = self.object_ref(id as CardId);
            if o.zone != Zone::Battlefield
                || o.controller != a.item.controller
                || o.tapped
                || !self.is_creature(id as CardId)
                || selected.contains(&object)
            {
                continue;
            }
            let mut next = selected.clone();
            next.push(object);
            if self.teamwork_prefix_fundable(&next, false)
                && emit(Action::ChooseTeammate { object })
            {
                return true;
            }
        }
        self.teamwork_prefix_fundable(selected, true) && emit(Action::FinishTeamwork)
    }
    pub(super) fn choose_teammate(&mut self, object: ObjectRef) {
        self.state
            .announcing
            .as_mut()
            .unwrap()
            .cast_costs
            .as_mut()
            .unwrap()
            .teammates
            .push(object);
    }
    pub(super) fn finish_teamwork(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        let c = a.cast_costs.as_mut().unwrap();
        let active = c.active.take().unwrap();
        assert!(matches!(
            c.clauses[usize::from(active)].clause,
            CastCostClause::Teamwork(_)
        ));
        c.clauses[usize::from(active)].paid = true;
        let ids = c.teammates.iter().map(|r| r.card).collect::<Vec<_>>();
        let actor = a.item.controller;
        self.tap_objects_as(actor, &ids);
        // This event is distinct from merely becoming tapped. Queue after the
        // complete simultaneous cost; these triggers wait until casting finishes.
        for id in ids {
            let source = self.object_view(id);
            for (index, ability) in source.trigger_templates() {
                if matches!(
                    ability.condition.event(),
                    TriggerCondition::SelfPaysTeamwork
                ) {
                    self.queue_trigger(&source, index);
                }
            }
        }
        self.finish_announcement();
    }
    pub fn teamwork_choice(&self) -> Option<TeamworkChoiceView> {
        if !matches!(self.state.pending, Pending::TeamworkPayment { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let selected = &a.cast_costs.as_ref()?.teammates;
        let mut sum = saddle::PowerSum::default();
        for r in selected {
            sum.add(self.stats(r.card).power);
        }
        Some(TeamworkChoiceView {
            minimum: a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .teamwork()?
                .minimum,
            selected: selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.teamwork_prefix_fundable(selected, true),
        })
    }
}
