//! Non-targeting Crew costs and sparse exact-incarnation layer-4 animation.
use super::saddle::PowerSum;
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewContext {
    pub contributors: Vec<ObjectRef>,
    /// None means the exact signed sum is outside i128, not that it was clamped.
    pub paid_power: Option<i128>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CrewContext {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.contributors.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewHistory {
    pub vehicle: ObjectRef,
    /// Distinct exact creatures that paid a Crew cost this turn, even if countered.
    pub contributors: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CrewPayment {
    pub source: ObjectRef,
    pub actor: Player,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub complete: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
pub(super) struct ContributorSelection {
    pub source: ObjectRef,
    pub actor: Player,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub complete: bool,
    pub saddle: bool,
    pub source_subtype: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ContributorSelection {
    pub(super) fn payable<W: ability_world::CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
    ) -> bool {
        let mut sum = PowerSum::default();
        if self
            .selected
            .iter()
            .any(|&r| !world.crew_saddle_eligible(self.source, self.actor, r))
        {
            return false;
        }
        for id in 0..world.game().state.objects.len() {
            let id = id as CardId;
            let reference = world.reference(id);
            if !world.crew_saddle_eligible(self.source, self.actor, reference) {
                continue;
            }
            let selected = self.selected.contains(&reference);
            let subject_matches = world.crew_saddle_subject_matches(
                self.source,
                if self.saddle { "Mount" } else { "Vehicle" },
                self.source_subtype,
            );
            let (base, bonus) = world.crew_saddle_contribution_terms(id, subject_matches);
            let positive = base > 0 || base + bonus > 0;
            // During selection this is a bound, not a choice of a hidden subset.
            // It is reevaluated in each exact reachable payment world.
            if selected || (!self.complete && positive) {
                sum.add(base);
                sum.add(bonus);
            }
        }
        sum.enough(self.minimum)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CrewState {
    pub animated: Vec<ObjectRef>,
    pub history: Vec<CrewHistory>,
    pub payment: Option<Box<CrewPayment>>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CrewState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.animated.len() * std::mem::size_of::<ObjectRef>()
            + self.history.len() * std::mem::size_of::<CrewHistory>()
            + self
                .history
                .iter()
                .map(|h| h.contributors.len())
                .sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
            + self.payment.as_ref().map_or(0, |p| {
                std::mem::size_of::<CrewPayment>()
                    + p.selected.len() * std::mem::size_of::<ObjectRef>()
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ObjectView {
    pub fn is_creature(&self) -> bool {
        self.kinds & 1 != 0
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl DamageFacts {
    pub fn is_creature(self) -> bool {
        self.kinds & 1 != 0
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivatedAbility {
    pub(super) fn crew_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::Crew { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn contributor_source_facts(&self, source: ObjectRef) -> Option<DamageFacts> {
        self.state
            .announcing
            .iter()
            .map(|a| &a.item)
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter().map(|f| &f.announcement.item)),
            )
            .filter_map(|item| item.source)
            .find(|packet| packet.object == source)
            .map(|packet| packet.facts)
    }
    pub(super) fn contributor_sum(
        &self,
        source: ObjectRef,
        actor: Player,
        selected: &[ObjectRef],
        saddle: bool,
        maximum: bool,
        fallback: Option<DamageFacts>,
    ) -> PowerSum {
        use ability_world::CharacteristicWorld;
        let subtype = if saddle { "Mount" } else { "Vehicle" };
        let fallback = fallback
            .or_else(|| self.contributor_source_facts(source))
            .is_some_and(|facts| facts.has_subtype(subtype));
        let subject_matches = self.crew_saddle_subject_matches(source, subtype, fallback);
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let id = id as CardId;
            let object = self.object_ref(id);
            if !self.crew_saddle_eligible(source, actor, object) {
                continue;
            }
            let chosen = selected.contains(&object);
            let (base, bonus) = self.crew_saddle_contribution_terms(id, subject_matches);
            let positive = base > 0 || base + bonus > 0;
            if chosen || maximum && positive {
                sum.add(base);
                sum.add(bonus);
            }
        }
        sum
    }
    pub(super) fn contributor_requirement(
        &self,
        goal: &FundingGoal,
    ) -> Option<Box<ContributorSelection>> {
        let (source, ability, _, definition, _) = goal.source.as_ref()?;
        let rules = definition.activation(*ability);
        let (minimum, saddle) = rules
            .crew_minimum()
            .map(|n| (n, false))
            .or_else(|| rules.saddle_minimum().map(|n| (n, true)))?;
        let mut selection = ContributorSelection {
            source: *source,
            actor: goal.actor,
            minimum,
            saddle,
            source_subtype: self.contributor_source_facts(*source).map_or_else(
                || {
                    self.on_battlefield(source.card)
                        && self.object_ref(source.card) == *source
                        && self.has_subtype(source.card, if saddle { "Mount" } else { "Vehicle" })
                },
                |facts| facts.has_subtype(if saddle { "Mount" } else { "Vehicle" }),
            ),
            selected: Vec::new(),
            complete: false,
        };
        if saddle {
            if let Some(p) = self
                .state
                .saddle_payment
                .as_deref()
                .filter(|p| p.source == *source && p.actor == goal.actor)
            {
                selection.selected = p.selected.clone();
                selection.complete = p.complete;
            }
        } else if let Some(p) = self
            .state
            .crew
            .as_ref()
            .and_then(|c| c.payment.as_deref())
            .filter(|p| p.source == *source && p.actor == goal.actor)
        {
            selection.selected = p.selected.clone();
            selection.complete = p.complete;
        }
        Some(Box::new(selection))
    }
    /// One borrowed query serves real objects and speculative funding overlays.
    pub(super) fn object_kinds(&self, id: CardId, object: &Object) -> u16 {
        characteristic_layers::kinds(self, id, object)
    }

    pub(super) fn current_kinds(&self, id: CardId) -> u16 {
        self.object_kinds(id, &self.state.objects[id as usize])
    }
    pub(super) fn is_creature(&self, id: CardId) -> bool {
        self.current_kinds(id) & 1 != 0
    }
    fn crew_eligible(&self, source: ObjectRef, actor: Player, r: ObjectRef) -> bool {
        r != source
            && self.state.objects.get(r.card as usize).is_some_and(|o| {
                o.zone == Zone::Battlefield
                    && o.generation == r.generation
                    && o.controller == actor
                    && !o.tapped
                    && self.object_kinds(r.card, o) & 1 != 0
            })
    }
    #[cfg(feature = "test-support")]
    pub(super) fn crew_fundable(&self, source: ObjectRef, actor: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(source, actor, r) {
                sum.add(
                    self.crew_saddle_power(r.card, source.card, "Vehicle")
                        .max(0),
                );
                if sum.enough(minimum) {
                    return true;
                }
            }
        }
        false
    }
    fn crew_valid(&self) -> bool {
        let Some(p) = self.state.crew.as_ref().and_then(|c| c.payment.as_deref()) else {
            return false;
        };
        self.on_battlefield(p.source.card)
            && self.object_ref(p.source.card) == p.source
            && self.state.objects[p.source.card as usize].controller == p.actor
            && p.selected
                .iter()
                .all(|&r| self.crew_eligible(p.source, p.actor, r))
    }
    fn crew_sum(&self, maximum: bool) -> PowerSum {
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        self.contributor_sum(p.source, p.actor, &p.selected, false, maximum, None)
    }
    pub(super) fn crew_choice(&self, viewer: Player) -> Option<CrewChoiceView> {
        let viewer = self.choice_information_player(viewer);
        let p = self.state.crew.as_ref()?.payment.as_ref()?;
        if p.actor != viewer || self.semantic_actor() != Some(viewer) {
            return None;
        }
        let sum = self.crew_sum(false);
        Some(CrewChoiceView {
            source: p.source,
            minimum: p.minimum,
            selected: p.selected.clone(),
            selected_power: sum.represented(),
            can_finish: !p.complete
                && self.crew_valid()
                && self.contributor_selection_fundable(&p.selected, true),
        })
    }
    pub(super) fn visit_crew_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if !self.crew_valid() {
            return false;
        }
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(p.source, p.actor, r) && !p.selected.contains(&r) {
                let mut selected = p.selected.clone();
                selected.push(r);
                if self.contributor_selection_fundable(&selected, false)
                    && emit(Action::ChooseCrewer { object: r })
                {
                    return true;
                }
            }
        }
        self.contributor_selection_fundable(&p.selected, true) && emit(Action::FinishCrew)
    }
    pub(super) fn start_crew_payment(&mut self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a
            .item
            .source
            .expect("cost announcement has object source")
            .card
            .activation(ability)
            .crew_minimum()
        else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        if self
            .state
            .crew
            .as_ref()
            .and_then(|c| c.payment.as_ref())
            .is_some()
        {
            return false;
        }
        let state = self.state.crew.get_or_insert_with(Default::default);
        state.payment = Some(Box::new(CrewPayment {
            source: a.item.source.expect("Crew source").object,
            actor: a.item.controller,
            minimum,
            selected: Vec::new(),
            complete: false,
        }));
        self.state.pending = Pending::Crew {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn choose_crewer(&mut self, object: ObjectRef) {
        let p = self.state.crew.as_mut().unwrap().payment.as_mut().unwrap();
        p.selected
            .try_reserve(1)
            .capacity("Crew selection capacity exhausted");
        p.selected.push(object);
    }
    pub(super) fn finish_crew_payment(&mut self) {
        assert!(self.crew_valid());
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        assert!(self.contributor_selection_fundable(&p.selected, true));
        let actor = p.actor;
        let free = self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .expect("frozen Crew cost")
            == ManaCost {
                colored: [0; 6],
                generic: 0,
            };
        if free && self.crew_sum(false).enough(p.minimum) {
            let sum = self.crew_sum(false);
            let p = self.state.crew.as_mut().unwrap().payment.take().unwrap();
            let mut a = self.state.announcing.take().unwrap();
            let context = CrewContext {
                contributors: p.selected,
                paid_power: sum.represented(),
            };
            if let Some(phase) = self.pay_activation(&mut a.item, [0; 6], Counters::default()) {
                self.suspend_activation_payment(
                    a.item,
                    phase,
                    ActivationCostRoute::Printed,
                    activations::ActivationPaymentCompletion::Crew(context),
                );
                return;
            }
            self.finish_paid_crew(a.item, context);
            return;
        }
        self.state
            .crew
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap()
            .complete = true;
        self.state.pending = Pending::PayCosts { player: actor };
    }
    pub(super) fn take_paid_contributor_completion(
        &mut self,
        item: &SpellView,
    ) -> Option<activations::ActivationPaymentCompletion> {
        let source = item.source?;
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return None;
        };
        let rules = source.card.activation(ability);
        if rules.crew_minimum().is_some()
            && self
                .state
                .crew
                .as_ref()
                .and_then(|c| c.payment.as_deref())
                .is_some_and(|p| {
                    p.complete && p.source == source.object && p.actor == item.controller
                })
        {
            let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
            let sum = self.contributor_sum(
                p.source,
                p.actor,
                &p.selected,
                false,
                false,
                Some(source.facts),
            );
            let p = self.state.crew.as_mut().unwrap().payment.take().unwrap();
            assert!(sum.enough(p.minimum));
            return Some(activations::ActivationPaymentCompletion::Crew(
                CrewContext {
                    contributors: p.selected,
                    paid_power: sum.represented(),
                },
            ));
        }
        if rules.saddle_minimum().is_some()
            && self.state.saddle_payment.as_deref().is_some_and(|p| {
                p.complete && p.source == source.object && p.actor == item.controller
            })
        {
            let p = self.state.saddle_payment.take().unwrap();
            return Some(activations::ActivationPaymentCompletion::Saddle(p.selected));
        }
        None
    }

    pub(super) fn finish_paid_crew(&mut self, mut item: SpellView, context: CrewContext) {
        let state = self.state.crew.as_mut().unwrap();
        let at = state
            .history
            .iter()
            .position(|h| {
                h.vehicle
                    == item
                        .source
                        .expect("cost announcement has object source")
                        .object
            })
            .unwrap_or_else(|| {
                state
                    .history
                    .try_reserve(1)
                    .capacity("Crew history capacity exhausted");
                state.history.push(CrewHistory {
                    vehicle: item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    contributors: Vec::new(),
                });
                state.history.len() - 1
            });
        let history = &mut state.history[at].contributors;
        for &r in &context.contributors {
            if !history.contains(&r) {
                history
                    .try_reserve(1)
                    .capacity("Crew history capacity exhausted");
                history.push(r)
            }
        }
        self.tap_objects(
            item.controller,
            &context
                .contributors
                .iter()
                .map(|r| r.card)
                .collect::<Vec<_>>(),
        );
        let participants = self.capture_crew_saddle_participants(&context.contributors);
        self.queue_crew_saddle_participant_triggers(
            participants,
            item.source.expect("Crew has an object source").object,
        );
        item.crew = Some(Box::new(context));
        self.state.priority = item.controller;
        self.push_stack_item(item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn crewed_vehicle_attack_triggers(&mut self, declared: &[AttackDeclaration]) {
        if self
            .state
            .crew
            .as_ref()
            .is_none_or(|state| state.history.is_empty())
        {
            return;
        }
        for attack in declared {
            let vehicle = self.object_ref(attack.card);
            let Some(history) = self
                .state
                .crew
                .as_ref()
                .unwrap()
                .history
                .iter()
                .find(|row| row.vehicle == vehicle)
            else {
                continue;
            };
            if !self.has_subtype(attack.card, "Vehicle") {
                continue;
            }
            // The paid cost relates two incarnations, regardless of who
            // controls either one now or whether the activation resolved.
            let observers: Vec<_> = history
                .contributors
                .iter()
                .filter(|crewer| {
                    self.on_battlefield(crewer.card)
                        && self.object_ref(crewer.card) == **crewer
                        && self.state.objects[crewer.card as usize]
                            .card
                            .triggers()
                            .iter()
                            .any(|ability| {
                                ability.condition.event() == TriggerCondition::CrewedVehicleAttacks
                            })
                })
                .map(|crewer| self.object_view(crewer.card))
                .collect();
            for source in observers {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if ability.condition.event() == TriggerCondition::CrewedVehicleAttacks
                        && self.queue_attack_trigger_from(&source, index, None)
                    {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .refer_objects([vehicle]);
                    }
                }
            }
        }
    }
    pub(super) fn forget_vehicle_animation(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.crew
            && let Ok(at) = state.animated.binary_search(&object)
        {
            state.animated.remove(at);
        }
    }
    pub(super) fn expire_vehicle_animation(&mut self) -> bool {
        let Some(state) = &mut self.state.crew else {
            return false;
        };
        let changed = !state.animated.is_empty();
        state.animated.clear();
        changed
    }
    pub(super) fn clear_crew_history(&mut self) {
        if let Some(state) = &mut self.state.crew {
            state.history.clear();
            if state.animated.is_empty() && state.payment.is_none() {
                self.state.crew = None;
            }
        }
    }
    pub(super) fn resolve_crew(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        self.animate_source(
            item,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
        );
        let state = self.state.crew.get_or_insert_with(Default::default);
        if let Err(at) = state.animated.binary_search(&source.object) {
            state
                .animated
                .try_reserve(1)
                .capacity("Vehicle animation capacity exhausted");
            state.animated.insert(at, source.object)
        }
    }
}
