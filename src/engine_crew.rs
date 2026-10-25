//! Non-targeting Crew costs and sparse exact-incarnation layer-4 animation.
use super::saddle::PowerSum;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewContext {
    pub contributors: Vec<ObjectRef>,
    /// None means the exact signed sum is outside i128, not that it was clamped.
    pub paid_power: Option<i128>,
}
impl CrewContext {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.contributors.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewHistory {
    pub vehicle: ObjectRef,
    /// Distinct exact creatures that paid a Crew cost this turn, even if countered.
    pub contributors: Vec<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CrewPayment {
    minimum: u16,
    selected: Vec<ObjectRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CrewState {
    pub animated: Vec<ObjectRef>,
    pub history: Vec<CrewHistory>,
    pub payment: Option<Box<CrewPayment>>,
}
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
impl ObjectView {
    pub fn is_creature(&self) -> bool {
        self.kinds & 1 != 0
    }
}
impl DamageFacts {
    pub fn is_creature(self) -> bool {
        self.kinds & 1 != 0
    }
}
impl ActivatedAbility {
    pub(super) fn crew_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::Crew { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
impl Game {
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
    pub(super) fn crew_fundable(&self, source: ObjectRef, actor: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(source, actor, r) {
                sum.add(self.stats(r.card).power.max(0));
                if sum.enough(minimum) {
                    return true;
                }
            }
        }
        false
    }
    fn crew_valid(&self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let Some(p) = self.state.crew.as_ref().and_then(|c| c.payment.as_ref()) else {
            return false;
        };
        self.state
            .objects
            .get(a.item.card as usize)
            .is_some_and(|o| {
                o.zone == Zone::Battlefield
                    && o.generation == a.item.object.generation
                    && o.controller == a.item.controller
            })
            && p.selected
                .iter()
                .all(|&r| self.crew_eligible(a.item.object, a.item.controller, r))
    }
    fn crew_sum(&self, maximum: bool) -> PowerSum {
        let a = self.state.announcing.as_ref().unwrap();
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(a.item.object, a.item.controller, r) {
                let selected = p.selected.contains(&r);
                let power = self.stats(r.card).power;
                if selected || maximum && power > 0 {
                    sum.add(power)
                }
            }
        }
        sum
    }
    pub(super) fn crew_choice(&self, viewer: Player) -> Option<CrewChoiceView> {
        let viewer = self.choice_information_player(viewer);
        if !matches!(self.state.pending,Pending::Crew{player} if player==viewer) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let p = self.state.crew.as_ref()?.payment.as_ref()?;
        let sum = self.crew_sum(false);
        Some(CrewChoiceView {
            source: a.item.object,
            minimum: p.minimum,
            selected: p.selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.crew_valid() && sum.enough(p.minimum),
        })
    }
    pub(super) fn visit_crew_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if !self.crew_valid() {
            return false;
        }
        let a = self.state.announcing.as_ref().unwrap();
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        let maximum = self.crew_sum(true);
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(a.item.object, a.item.controller, r) && !p.selected.contains(&r) {
                let mut possible = maximum;
                let power = self.stats(r.card).power;
                if power < 0 {
                    possible.add(power)
                }
                if possible.enough(p.minimum) && emit(Action::ChooseCrewer { object: r }) {
                    return true;
                }
            }
        }
        self.crew_sum(false).enough(p.minimum) && emit(Action::FinishCrew)
    }
    pub(super) fn start_crew_payment(&mut self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a.item.source_card.activation(ability).crew_minimum() else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        let state = self.state.crew.get_or_insert_with(Default::default);
        assert!(state.payment.is_none());
        state.payment = Some(Box::new(CrewPayment {
            minimum,
            selected: Vec::new(),
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
        let sum = self.crew_sum(false);
        let p = self.state.crew.as_mut().unwrap().payment.take().unwrap();
        assert!(sum.enough(p.minimum));
        let mut a = self.state.announcing.take().unwrap();
        let context = CrewContext {
            contributors: p.selected,
            paid_power: sum.represented(),
        };
        let state = self.state.crew.as_mut().unwrap();
        let at = state
            .history
            .iter()
            .position(|h| h.vehicle == a.item.object)
            .unwrap_or_else(|| {
                state
                    .history
                    .try_reserve(1)
                    .capacity("Crew history capacity exhausted");
                state.history.push(CrewHistory {
                    vehicle: a.item.object,
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
        self.pay_activation(&mut a.item, [0; 6], Counters::default());
        self.tap_objects(
            &context
                .contributors
                .iter()
                .map(|r| r.card)
                .collect::<Vec<_>>(),
        );
        a.item.crew = Some(Box::new(context));
        self.state.priority = a.item.controller;
        self.push_stack_item(a.item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
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
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
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
        if let Err(at) = state.animated.binary_search(&item.object) {
            state
                .animated
                .try_reserve(1)
                .capacity("Vehicle animation capacity exhausted");
            state.animated.insert(at, item.object)
        }
    }
}
