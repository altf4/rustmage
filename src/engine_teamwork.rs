//! Optional teamwork declarations and simultaneous, non-targeting creature-tap costs.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TeamworkChoiceView {
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}

impl Game {
    pub(super) fn teamwork_initial_fundable(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> bool {
        let card = self.state.objects[id as usize].card;
        let spec = card.teamwork().unwrap();
        let normal_targets = match card.effect() {
            Effect::Spell(plan, _) => self.cast_targets_available(plan, actor, self.object_ref(id)),
            Effect::Modal(modes) => modes.iter().enumerate().any(|(i, m)| {
                i < 64
                    && spec.normal_modes & (1u64 << i) != 0
                    && self.cast_targets_available(m.targets, actor, self.object_ref(id))
            }),
            _ => true,
        };
        if normal_targets {
            return self.spell_can_fund(actor, id, base);
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
            &[CastCostClause::Teamwork(spec.minimum)],
        )
        .unwrap_or(false)
    }
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
    pub(super) fn teamwork_intent_fundable(&self, pay: bool) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        a.item.cast.as_mut().unwrap().teamwork = Some(pay);
        a.initialize_cast_costs(improvise::potential(
            self,
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
        ));
        let available = if let Some(modes) = a.item.modes() {
            modes.iter().enumerate().any(|(i, m)| {
                self.teamwork_mode_allowed(&a.item, i)
                    && self.item_targets_available(&a.item, m.targets)
            })
        } else {
            let plan = self.target_spec(&a.item);
            self.item_targets_available(&a.item, plan)
        };
        if !available {
            return false;
        }
        self.teamwork_proposal_fundable(&a)
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
        self.tap_objects(&ids);
        // This event is distinct from merely becoming tapped. Queue after the
        // complete simultaneous cost; these triggers wait until casting finishes.
        for id in ids {
            let source = self.object_view(id);
            for (index, ability) in source.card.triggers().iter().enumerate() {
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
