//! Complete exact-counter allocation domains with bounded indexed representatives.
use super::*;

/// Every allocation contained here removes those counters simultaneously from
/// the exact recipient. Named kinds are included, and totals use u128.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CounterRemovalDomain {
    pub recipient: ObjectRef,
    pub available: Counters,
    pub minimum: u16,
    pub maximum: Option<u64>,
}
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
impl Game {
    pub fn counter_removal_domain(&self) -> Option<CounterRemovalDomain> {
        if self.state.outcome.is_some()
            || !matches!(self.state.pending, Pending::EffectChoice { .. })
        {
            return None;
        }
        let domain = self
            .state
            .resolving
            .as_ref()?
            .choice
            .as_ref()?
            .counter_removal
            .as_deref()?;
        (self.on_battlefield(domain.recipient.card)
            && self.object_ref(domain.recipient.card) == domain.recipient)
            .then(|| domain.clone())
    }
    pub(super) fn counter_removal_valid(&self, counters: &Counters) -> bool {
        matches!(self.state.pending, Pending::EffectChoice { .. })
            && self
                .state
                .resolving
                .as_ref()
                .and_then(|r| r.choice.as_ref())
                .and_then(|c| c.counter_removal.as_deref())
                .is_some_and(|domain| {
                    self.on_battlefield(domain.recipient.card)
                        && self.object_ref(domain.recipient.card) == domain.recipient
                        && domain.contains(counters)
                        && self
                            .counter_amounts(domain.recipient.card)
                            .contains(counters)
                })
    }
    pub(super) fn start_counter_removal(
        &mut self,
        item: &SpellView,
        subject: Subject,
        minimum: u16,
        maximum: Option<u64>,
        optional_skip: Option<usize>,
    ) -> bool {
        assert!(maximum.is_none_or(|n| n >= u64::from(minimum)));
        let recipient = match subject {
            Subject::Source => (self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object)
                .then_some(item.object),
            Subject::Attached => self.attached_permanent(item).map(|id| self.object_ref(id)),
            Subject::Entering => self.entering_subject(item).map(|id| self.object_ref(id)),
            Subject::Targets => item.targets.iter().find_map(|bound| {
                bound.object.filter(|object| {
                    self.same_target_object(*bound) && self.on_battlefield(object.card)
                })
            }),
        };
        let pc = self.state.resolving.as_ref().unwrap().pc;
        let skip_to = optional_skip.map(|skip| pc.checked_add(skip).expect("program offset"));
        if let Some(skip) = skip_to {
            assert!(skip <= self.state.resolving.as_ref().unwrap().program.len());
        }
        let Some(recipient) = recipient
            .filter(|object| self.counter_amounts(object.card).total() >= u128::from(minimum))
        else {
            if let Some(skip) = skip_to {
                self.state.resolving.as_mut().unwrap().pc = skip;
            }
            return false;
        };
        let domain = CounterRemovalDomain {
            recipient,
            available: self.counter_amounts(recipient.card).to_owned(),
            minimum,
            maximum,
        };
        self.state.resolving.as_mut().unwrap().choice =
            Some(Box::new(effect_choices::EffectChoice {
                kind: EffectChoiceKind::RemoveCounters,
                filter: evidence::cost(0).filter,
                amount: minimum,
                source: item.object,
                required: [0; 2],
                selected: std::array::from_fn(|_| Vec::new()),
                skip_to,
                greatest_power: [None; 2],
                discard_mana_value_life: None,
                hand_defender: None,
                hand_creature_entry: None,
                entry_life_bonus: None,
                hand_reveal: None,
                blight_copy_attacking: false,
                counter_removal: Some(Box::new(domain)),
                proliferate: None,
            }));
        self.state.pending = Pending::EffectChoice {
            player: item.controller,
        };
        true
    }
    pub(super) fn choose_counter_removal(&mut self, counters: &Counters) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let domain = choice.counter_removal.unwrap();
        self.pay_counters(domain.recipient.card, counters);
        self.run_program();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counter_removal_complete_domain_includes_all_named_kinds_and_wide_totals() {
        let mut available = Counters::default();
        for kind in CounterKind::ALL {
            available.set(kind, u64::MAX);
        }
        let domain = CounterRemovalDomain {
            recipient: ObjectRef {
                card: 0,
                generation: 1,
            },
            available: available.clone(),
            minimum: 0,
            maximum: None,
        };
        assert!(domain.contains(&available));
        for kind in CounterKind::ALL {
            assert!(domain.contains(&Counters::of(kind, u64::MAX - 19)));
        }
        let action = Action::ChooseCounterRemoval {
            counters: available,
        };
        assert_eq!(
            serde_json::from_value::<Action>(serde_json::to_value(&action).unwrap()).unwrap(),
            action
        );
        let mut actions = Vec::new();
        domain.representatives(&mut |counters| {
            actions.push(counters);
            false
        });
        assert!(actions.len() <= CounterKind::ALL.len() * 2 + 3);
        assert!(actions.iter().all(|c| domain.contains(c)));
    }
    #[test]
    fn counter_removal_small_domains_expose_mixed_subsets_and_exact_one_rejects_zero_or_two() {
        let mut domain = CounterRemovalDomain {
            recipient: ObjectRef {
                card: 0,
                generation: 1,
            },
            available: Counters::of(CounterKind::Soul, 2).with(CounterKind::Stun, 2),
            minimum: 0,
            maximum: None,
        };
        let mut actions = Vec::new();
        domain.representatives(&mut |counters| {
            actions.push(counters);
            false
        });
        assert_eq!(actions.len(), 9);
        assert!(actions.contains(&Counters::of(CounterKind::Soul, 1).with(CounterKind::Stun, 1)));
        assert!(!domain.contains(&Counters::of(CounterKind::Soul, 3)));
        domain.minimum = 1;
        domain.maximum = Some(1);
        assert!(!domain.contains(&Counters::default()));
        assert!(!domain.contains(&Counters::of(CounterKind::Soul, 2)));
        assert!(domain.contains(&Counters::of(CounterKind::Soul, 1)));
        actions.clear();
        domain.representatives(&mut |counters| {
            actions.push(counters);
            false
        });
        assert_eq!(actions.len(), 2);
    }
}
