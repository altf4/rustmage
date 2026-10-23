//! Incremental non-targeting tap costs and exact, resolved Saddle designations.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SaddlePayment {
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
}
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
#[derive(Clone, Copy, Default)]
pub(super) struct PowerSum {
    positive: (u64, u128),
    negative: (u64, u128),
}
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
impl ActivatedAbility {
    pub(super) fn saddle_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::BecomeSaddled { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
impl Game {
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
    pub(super) fn saddle_fundable(&self, source: ObjectRef, p: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut available = 0u16;
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(source, p, r) {
                let power = self.stats(r.card).power.max(0);
                available = available.saturating_add(u16::try_from(power).unwrap_or(u16::MAX));
                if available >= minimum {
                    return true;
                }
            }
        }
        false
    }
    fn saddle_frame_valid(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let Some(f) = self.state.saddle_payment.as_ref() else {
            return false;
        };
        self.state
            .objects
            .get(a.item.object.card as usize)
            .is_some_and(|o| {
                o.generation == a.item.object.generation
                    && o.zone == Zone::Battlefield
                    && o.controller == a.item.controller
            })
            && f.selected
                .iter()
                .all(|r| self.saddle_eligible(a.item.object, a.item.controller, *r))
    }
    fn saddle_sums(&self, maximum: bool) -> PowerSum {
        let a = self.state.announcing.as_ref().unwrap();
        let f = self.state.saddle_payment.as_ref().unwrap();
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(a.item.object, a.item.controller, r) {
                let selected = f.selected.contains(&r);
                if selected || maximum {
                    let p = self.stats(r.card).power;
                    if selected || p > 0 {
                        sum.add(p);
                    }
                }
            }
        }
        sum
    }
    pub fn saddle_choice(&self) -> Option<SaddleChoiceView> {
        if !matches!(self.state.pending, Pending::Saddle { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = self.state.saddle_payment.as_ref()?;
        let sum = self.saddle_sums(false);
        Some(SaddleChoiceView {
            source: a.item.object,
            minimum: f.minimum,
            selected: f.selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.saddle_frame_valid() && sum.enough(f.minimum),
        })
    }
    pub(super) fn visit_saddle_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = self.state.saddle_payment.as_ref().unwrap();
        if !self.saddle_frame_valid() {
            return false;
        }
        let maximum = self.saddle_sums(true);
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.saddle_eligible(a.item.object, a.item.controller, object)
                || f.selected.contains(&object)
            {
                continue;
            }
            let p = self.stats(object.card).power;
            let mut possible = maximum;
            if p < 0 {
                possible.add(p);
            }
            if possible.enough(f.minimum) && emit(Action::ChooseSaddler { object }) {
                return true;
            }
        }
        self.saddle_sums(false).enough(f.minimum) && emit(Action::FinishSaddle)
    }
    pub(super) fn start_saddle_payment(&mut self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a.item.source_card.activation(ability).saddle_minimum() else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        assert!(self.state.saddle_payment.is_none());
        self.state.saddle_payment = Some(Box::new(SaddlePayment {
            minimum,
            selected: Vec::new(),
        }));
        self.state.pending = Pending::Saddle {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn finish_saddle_payment(&mut self) {
        assert!(self.saddle_frame_valid());
        let f = self.state.saddle_payment.take().unwrap();
        let mut a = self.state.announcing.take().unwrap();
        assert!(f.selected.iter().all(|r| self.saddle_eligible(
            a.item.object,
            a.item.controller,
            *r
        )));
        let mut total = PowerSum::default();
        for r in &f.selected {
            total.add(self.stats(r.card).power);
        }
        assert!(total.enough(f.minimum));
        // All powers are checked before the simultaneous tap cost. Tap observers
        // queue during payment, then are ordered above the activated ability.
        self.pay_activation(&mut a.item, [0; 6], Counters::default());
        self.tap_objects(&f.selected.iter().map(|r| r.card).collect::<Vec<_>>());
        self.state.priority = a.item.controller;
        self.push_stack_item(a.item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn resolve_saddle(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let first = !self.is_saddled(item.object);
        if first {
            self.state.saddled.push(item.object);
        }
        if first {
            let source = self.object_view(item.card);
            for (index, a) in source.card.triggers().iter().enumerate() {
                if matches!(a.condition.event(), TriggerCondition::SelfSaddledFirst) {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
