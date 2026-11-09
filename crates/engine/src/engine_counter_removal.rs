//! Complete exact-counter allocation domains with bounded indexed representatives.
use super::*;

/// Every allocation contained here removes those counters simultaneously from
/// the exact recipient. Named kinds are included, and totals use u128.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub struct CounterRemovalDomain {
    pub recipient: ObjectRef,
    pub available: Counters,
    pub minimum: u16,
    pub maximum: Option<u64>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CounterRemovalDomain {
    pub fn contains(&self, counters: &Counters) -> bool {
        let total = CounterAmounts::from(counters).total();
        CounterAmounts::from(&self.available).contains(counters)
            && total >= u128::from(self.minimum)
            && self
                .maximum
                .is_none_or(|maximum| total <= u128::from(maximum))
    }
    pub(super) fn representatives(&self, emit: &mut impl FnMut(Counters) -> bool) -> bool {
        let mut size = 1u128;
        for kind in CounterKind::ALL {
            size = size.saturating_mul(u128::from(self.available.get(kind)) + 1);
            if size > 64 {
                break;
            }
        }
        if size <= 64 {
            fn enumerate(
                domain: &CounterRemovalDomain,
                index: usize,
                counters: &mut Counters,
                emit: &mut impl FnMut(Counters) -> bool,
            ) -> bool {
                let Some(&kind) = CounterKind::ALL.get(index) else {
                    return domain.contains(counters) && emit(counters.clone());
                };
                for amount in 0..=domain.available.get(kind) {
                    counters.set(kind, amount);
                    if enumerate(domain, index + 1, counters, emit) {
                        return true;
                    }
                }
                counters.set(kind, 0);
                false
            }
            return enumerate(self, 0, &mut Counters::default(), emit);
        }
        let mut candidates = vec![Counters::default(), self.available.clone()];
        let mut minimum = u64::from(self.minimum);
        let mut greedy = Counters::default();
        for kind in CounterKind::ALL {
            let available = self.available.get(kind);
            if available == 0 {
                continue;
            }
            candidates.push(Counters::of(kind, 1));
            candidates.push(Counters::of(
                kind,
                self.maximum.map_or(available, |n| n.min(available)),
            ));
            let amount = minimum.min(available);
            greedy.set(kind, amount);
            minimum -= amount;
        }
        candidates.push(greedy);
        for (index, counters) in candidates.iter().enumerate() {
            if self.contains(counters)
                && !candidates[..index].contains(counters)
                && emit(counters.clone())
            {
                return true;
            }
        }
        false
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub fn counter_removal_domain(&self) -> Option<CounterRemovalDomain> {
        if self.state.outcome.is_some()
            || !matches!(self.state.pending, Pending::TargetCounterRemoval { .. })
        {
            return None;
        }
        let choice = self
            .state
            .resolving
            .as_ref()?
            .target_counter_removal
            .as_ref()?;
        (self.on_battlefield(choice.target.card)
            && self.object_ref(choice.target.card) == choice.target)
            .then(|| CounterRemovalDomain {
                recipient: choice.target,
                available: self.counter_amounts(choice.target.card).to_owned(),
                minimum: 0,
                maximum: choice.maximum,
            })
    }
    pub(super) fn finish_target_counter_removal(&mut self) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .target_counter_removal
            .take()
            .unwrap();
        if self.on_battlefield(choice.target.card)
            && self.object_ref(choice.target.card) == choice.target
        {
            self.pay_counters(choice.target.card, &choice.selected);
            if !choice.selected.is_empty() {
                let resolution = self.state.resolving.as_ref().unwrap();
                if let Some(Instruction::MayRemoveAnySourceCounterReflexive(ability)) =
                    resolution.program.get(resolution.pc - 1)
                {
                    let ability = *ability;
                    let item = resolution.item.clone();
                    self.queue_reflexive(&item, ability, None);
                }
            }
        }
        self.run_program();
    }
}
