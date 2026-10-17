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
        let mut kinds = object.card.kind_mask();
        if object.zone == Zone::Battlefield
            && self.state.crew.as_ref().is_some_and(|c| {
                c.animated.contains(&ObjectRef {
                    card: id,
                    generation: object.generation,
                })
            })
        {
            kinds |= 17;
        }
        kinds
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
                    && o.owner == actor
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
                    && o.owner == a.item.controller
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
        let Some(minimum) = a.item.source_card.activations()[ability as usize].crew_minimum()
        else {
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
        self.pay_activation(&a.item, [0; 6], Counters::default());
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
    pub(super) fn resolve_crew(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let state = self.state.crew.get_or_insert_with(Default::default);
        if !state.animated.contains(&item.object) {
            state
                .animated
                .try_reserve(1)
                .capacity("Vehicle animation capacity exhausted");
            state.animated.push(item.object)
        }
    }
}

#[cfg(test)]
mod core_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    #[test]
    fn core_live_kind_is_exact_and_printed_vehicle_identity_is_immutable() {
        let mut g = scenario();
        let card = Card::from_name("Air Response Unit").unwrap();
        assert!(!card.is_supported());
        let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
        let old = g.object_ref(id);
        assert!(!g.object_view(id).is_creature());
        assert_eq!(g.stats(id), Stats::default());
        g.state.crew = Some(Box::new(CrewState {
            animated: vec![old],
            ..Default::default()
        }));
        assert!(g.object_view(id).is_creature());
        assert!(!card.is_creature());
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 3));
        let captured = g.object_view(id);
        let facts = g.damage_facts(id);
        let saved = g.snapshot();
        let mut child = g.clone();
        child.move_object(id, Zone::Hand);
        assert!(!child.object_view(id).is_creature());
        assert!(captured.is_creature() && facts.is_creature());
        child.move_object(id, Zone::Battlefield);
        assert!(!child.object_view(id).is_creature());
        assert_ne!(child.object_ref(id), old);
        assert!(g.object_view(id).is_creature());
        child.restore(&saved).unwrap();
        assert!(child.object_view(id).is_creature());
    }
}
