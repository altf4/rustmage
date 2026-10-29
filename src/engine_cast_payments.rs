//! Actual cast payments and complete coupled numeric allocation domains.
use super::*;

/// Every six-vector `payment` satisfying these constraints is a legal PayCosts
/// allocation: required[i] <= payment[i] <= available[i], and the sum of
/// payment[i] - required[i] is exactly generic. Colorless is component five.
/// Numeric representatives in legal_actions do not restrict this complete space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaPaymentDomain {
    pub available: Mana,
    pub required: Mana,
    pub generic: u32,
}
impl ManaPaymentDomain {
    pub fn contains(&self, payment: Mana) -> bool {
        if (0..6).any(|i| payment[i] < self.required[i] || payment[i] > self.available[i]) {
            return false;
        }
        payment
            .into_iter()
            .zip(self.required)
            .map(|(p, r)| u64::from(p - r))
            .sum::<u64>()
            == u64::from(self.generic)
    }
    /// Exhaustive for spaces up to 64 points, bounded for larger numeric spaces.
    pub(super) fn representatives(&self, mut emit: impl FnMut(Mana)) {
        fn visit(
            d: &ManaPaymentDomain,
            i: usize,
            left: u32,
            mut paid: Mana,
            count: &mut usize,
            emit: &mut impl FnMut(Mana),
        ) {
            if *count == 64 {
                return;
            }
            if i == 5 {
                if d.available[i] - d.required[i] >= left {
                    paid[i] += left;
                    *count += 1;
                    emit(paid);
                }
                return;
            }
            let spare = d.available[i] - d.required[i];
            // Prune impossible branches before iterating. Work is bounded by
            // six components and the number of emitted representatives.
            let remaining_capacity: u64 = (i + 1..6)
                .map(|j| u64::from(d.available[j] - d.required[j]))
                .sum();
            let minimum = u64::from(left).saturating_sub(remaining_capacity) as u32;
            for n in minimum..=spare.min(left) {
                paid[i] = d.required[i] + n;
                visit(d, i + 1, left - n, paid, count, emit);
                if *count == 64 {
                    break;
                }
            }
        }
        if (0..6).any(|i| self.available[i] < self.required[i]) {
            return;
        }
        visit(self, 0, self.generic, self.required, &mut 0, &mut emit);
    }
}
impl CastInfo {
    pub fn mana_spent(self) -> u64 {
        self.payment.into_iter().map(u64::from).sum()
    }
    pub fn colors_spent(self) -> u8 {
        (0..5).filter(|&i| self.payment[i] > 0).count() as u8
    }
}

/// Frozen identity and actual payment of the completed cast that caused a trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastPaymentEvent {
    pub mana_value: u64,
    pub stack_id: u64,
    pub object: ObjectRef,
    pub card: Card,
    pub controller: Player,
    pub payment: Mana,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub treasure_mana_spent: bool,
}
impl SpellView {
    /// Mana spent is a fact of an actual cast, never inherited by a spell copy.
    pub(super) fn actual_cast_event(&self) -> Option<CastPaymentEvent> {
        self.cast
            .filter(|_| !self.is_copy)
            .zip(self.source)
            .map(|(cast, source)| CastPaymentEvent {
                mana_value: self.mana_value(),
                stack_id: self.id,
                object: source.object,
                card: source.card,
                controller: cast.caster,
                payment: cast.payment,
                treasure_mana_spent: cast.treasure_mana_spent,
            })
    }
}
impl Game {
    pub(super) fn item_cast_payment(&self, item: &SpellView) -> Mana {
        item.cast_event
            .as_ref()
            .map(|e| e.payment)
            .or_else(|| item.cast.map(|c| c.payment))
            .unwrap_or([0; 6])
    }
    pub(super) fn cast_payment_condition(
        &self,
        condition: TriggerCondition,
        source: ObjectRef,
        event: Option<&CastPaymentEvent>,
    ) -> bool {
        if condition.requires_treasure_mana()
            && !event.is_some_and(|event| event.treasure_mana_spent)
        {
            return false;
        }
        let Some((minimum, increment)) = condition.mana_spent_rule() else {
            return true;
        };
        let Some(event) = event else { return false };
        let spent = event.payment.into_iter().map(u64::from).sum::<u64>();
        if spent < minimum {
            return false;
        }
        if !increment {
            return true;
        }
        self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
            && self.is_creature(source.card)
            && {
                let stats = self.stats(source.card);
                i128::from(spent) > stats.power || i128::from(spent) > stats.toughness
            }
    }
}
