//! Incremental non-targeting tap costs and exact, resolved Saddle designations.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SaddlePayment {
    pub source: ObjectRef,
    pub actor: Player,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub complete: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SaddleChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    /// None only if the exact signed aggregate is outside i128; legality stays exact.
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
/// Each sign sums magnitudes with a u64 carry above u128. CardId bounds the
/// number of terms, so the high limb cannot overflow. Cancellation is exact.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Default)]
pub(super) struct PowerSum {
    positive: (u64, u128),
    negative: (u64, u128),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PowerSum {
    pub(super) fn add(&mut self, p: i128) {
        let limb = if p < 0 {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let (low, carry) = limb.1.overflowing_add(p.unsigned_abs());
        limb.1 = low;
        limb.0 += u64::from(carry);
    }
    fn difference(self) -> (bool, u64, u128) {
        let neg = self.negative > self.positive;
        let (a, b) = if neg {
            (self.negative, self.positive)
        } else {
            (self.positive, self.negative)
        };
        let (low, borrow) = a.1.overflowing_sub(b.1);
        (neg, a.0 - b.0 - u64::from(borrow), low)
    }
    pub(super) fn enough(self, n: u16) -> bool {
        let (neg, hi, lo) = self.difference();
        !neg && (hi > 0 || lo >= u128::from(n))
    }
    pub(super) fn represented(self) -> Option<i128> {
        let (neg, hi, lo) = self.difference();
        if hi > 0 {
            return None;
        }
        if neg {
            if lo == 1u128 << 127 {
                Some(i128::MIN)
            } else {
                i128::try_from(lo).ok().map(|n| -n)
            }
        } else {
            i128::try_from(lo).ok()
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivatedAbility {
    pub(super) fn saddle_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::BecomeSaddled { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    #[cfg(feature = "test-support")]
    pub(super) fn crew_saddle_power(&self, card: CardId, subject: CardId, subtype: &str) -> i128 {
        ability_world::CharacteristicWorld::crew_saddle_contribution(self, card, subject, subtype)
    }
    pub(super) fn is_saddled(&self, object: ObjectRef) -> bool {
        self.state.saddled.contains(&object)
    }
    fn saddle_eligible(&self, source: ObjectRef, p: Player, r: ObjectRef) -> bool {
        self.state.objects.get(r.card as usize).is_some_and(|o| {
            o.generation == r.generation
                && o.zone == Zone::Battlefield
                && o.controller == p
                && self.is_creature(r.card)
                && !o.tapped
                && r != source
        })
    }
    #[cfg(feature = "test-support")]
    pub(super) fn saddle_fundable(&self, source: ObjectRef, p: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut available = 0u16;
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(source, p, r) {
                let power = self.crew_saddle_power(r.card, source.card, "Mount").max(0);
                available = available.saturating_add(u16::try_from(power).unwrap_or(u16::MAX));
                if available >= minimum {
                    return true;
                }
            }
        }
        false
    }
    fn saddle_frame_valid(&self) -> bool {
        let Some(f) = self.state.saddle_payment.as_deref() else {
            return false;
        };
        self.on_battlefield(f.source.card)
            && self.object_ref(f.source.card) == f.source
            && self.state.objects[f.source.card as usize].controller == f.actor
            && f.selected
                .iter()
                .all(|&r| self.saddle_eligible(f.source, f.actor, r))
    }
    fn saddle_sums(&self, maximum: bool) -> PowerSum {
        let f = self.state.saddle_payment.as_ref().unwrap();
        self.contributor_sum(f.source, f.actor, &f.selected, true, maximum, None)
    }
    pub fn saddle_choice(&self) -> Option<SaddleChoiceView> {
        let f = self.state.saddle_payment.as_ref()?;
        if self.semantic_actor() != Some(f.actor) {
            return None;
        }
        let sum = self.saddle_sums(false);
        Some(SaddleChoiceView {
            source: f.source,
            minimum: f.minimum,
            selected: f.selected.clone(),
            selected_power: sum.represented(),
            can_finish: !f.complete
                && self.saddle_frame_valid()
                && self.contributor_selection_fundable(&f.selected, true),
        })
    }
    pub(super) fn visit_saddle_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let f = self.state.saddle_payment.as_ref().unwrap();
        if !self.saddle_frame_valid() {
            return false;
        }
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.saddle_eligible(f.source, f.actor, object) || f.selected.contains(&object) {
                continue;
            }
            let mut selected = f.selected.clone();
            selected.push(object);
            if self.contributor_selection_fundable(&selected, false)
                && emit(Action::ChooseSaddler { object })
            {
                return true;
            }
        }
        self.contributor_selection_fundable(&f.selected, true) && emit(Action::FinishSaddle)
    }
    pub(super) fn start_saddle_payment(&mut self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
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
            .saddle_minimum()
        else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        if self.state.saddle_payment.is_some() {
            return false;
        }
        self.state.saddle_payment = Some(Box::new(SaddlePayment {
            source: a.item.source.expect("Saddle source").object,
            actor: a.item.controller,
            minimum,
            selected: Vec::new(),
            complete: false,
        }));
        self.state.pending = Pending::Saddle {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn finish_saddle_payment(&mut self) {
        assert!(self.saddle_frame_valid());
        let f = self.state.saddle_payment.as_ref().unwrap();
        assert!(self.contributor_selection_fundable(&f.selected, true));
        let actor = f.actor;
        let free = self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .expect("frozen Saddle cost")
            == ManaCost {
                colored: [0; 6],
                generic: 0,
            };
        if free && self.saddle_sums(false).enough(f.minimum) {
            let f = self.state.saddle_payment.take().unwrap();
            let mut a = self.state.announcing.take().unwrap();
            if let Some(phase) = self.pay_activation(&mut a.item, [0; 6], Counters::default()) {
                self.suspend_activation_payment(
                    a.item,
                    phase,
                    ActivationCostRoute::Printed,
                    activations::ActivationPaymentCompletion::Saddle(f.selected),
                );
                return;
            }
            self.finish_paid_saddle(a.item, f.selected);
            return;
        }
        self.state.saddle_payment.as_mut().unwrap().complete = true;
        self.state.pending = Pending::PayCosts { player: actor };
    }
    pub(super) fn finish_paid_saddle(&mut self, item: SpellView, selected: Vec<ObjectRef>) {
        self.tap_objects(&selected.iter().map(|r| r.card).collect::<Vec<_>>());
        let participants = self.capture_crew_saddle_participants(&selected);
        self.queue_crew_saddle_participant_triggers(
            participants,
            item.source.expect("Saddle has an object source").object,
        );
        self.state.priority = item.controller;
        self.push_stack_item(item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn resolve_saddle(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        self.mark_saddled(source.object);
    }
    pub(super) fn mark_saddled_subject(&mut self, item: &SpellView, subject: Subject) {
        for id in self.keyword_subjects(item, subject) {
            self.mark_saddled(self.object_ref(id));
        }
    }
    fn mark_saddled(&mut self, recipient: ObjectRef) {
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        let first = !self.is_saddled(recipient);
        if first {
            self.state.saddled.push(recipient);
        }
        if first {
            let source = self.object_view(recipient.card);
            for (index, a) in source.card.triggers().iter().enumerate() {
                if matches!(a.condition.event(), TriggerCondition::SelfSaddledFirst) {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
