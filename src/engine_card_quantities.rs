//! Selected source-counter facts belong to owned ability packets, not hot objects.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceCounterQuantityView {
    pub kind: CounterKind,
    pub count: u64,
}

fn read_counter_kinds(program: &[Instruction], kinds: &mut Vec<CounterKind>) {
    fn amount(value: AmountValue, kinds: &mut Vec<CounterKind>) {
        match value {
            AmountValue::SourceCounterQuantity(kind) => {
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
            AmountValue::Multiple { value, .. } => amount(*value, kinds),
            AmountValue::Sum(values) => {
                for value in values {
                    amount(*value, kinds);
                }
            }
            _ => (),
        }
    }
    for instruction in program {
        match *instruction {
            Instruction::CreateTokensWithSourceCounters { .. }
            | Instruction::PutSourceCountersOnTargets => {
                for kind in CounterKind::ALL {
                    if !kinds.contains(&kind) {
                        kinds.push(kind);
                    }
                }
            }
            Instruction::Modal(modes) | Instruction::ModalOnce(modes) => {
                for mode in modes {
                    read_counter_kinds(mode.program, kinds);
                }
            }
            Instruction::SelectLibraryTo(effect) => {
                if let Some(value) = effect.maximum_mana_value {
                    amount(value, kinds);
                }
            }
            Instruction::IfAmountAtLeast { value, .. }
            | Instruction::PutLandCards { maximum: value, .. }
            | Instruction::DiscardScaled(_, value)
            | Instruction::AddCastEntryCounters { amount: value, .. }
            | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
            | Instruction::SetCreatureForm { base: value, .. }
            | Instruction::MoveCountersFromSource { amount: value, .. }
            | Instruction::DrawScaled(_, value)
            | Instruction::GainLifeScaled(_, value)
            | Instruction::LoseLifeScaled(_, value)
            | Instruction::DrainLifeScaled(value)
            | Instruction::DamageScaled(value)
            | Instruction::AddCombatRedScaled(value)
            | Instruction::Mobilize { amount: value, .. }
            | Instruction::CounterUnlessPayScaled { amount: value, .. }
            | Instruction::PutCountersScaled { amount: value, .. }
            | Instruction::PutPermanentCountersScaled { amount: value, .. }
            | Instruction::GrowScaled { amount: value, .. }
            | Instruction::GrowCreaturesScaled { amount: value, .. }
            | Instruction::CreateTokensScaled { amount: value, .. }
            | Instruction::CreateTokensWithCounters {
                counters: value, ..
            } => amount(value, kinds),
            _ => (),
        }
    }
}

#[allow(
    clippy::box_collection,
    reason = "Absent facts keep the common owned packet one pointer wide"
)]
pub(super) fn counter_quantity_facts(
    program: &[Instruction],
    count: impl Fn(CounterKind) -> u64,
) -> Option<Box<Vec<SourceCounterQuantityView>>> {
    let mut kinds = Vec::new();
    read_counter_kinds(program, &mut kinds);
    if kinds.is_empty() {
        return None;
    }
    Some(Box::new(
        kinds
            .into_iter()
            .map(|kind| SourceCounterQuantityView {
                kind,
                count: count(kind),
            })
            .collect(),
    ))
}

impl SpellView {
    pub(super) fn refresh_counter_quantities(&mut self, count: impl Fn(CounterKind) -> u64) {
        if let Some(facts) = &mut self.source_counter_lki {
            for fact in facts.iter_mut() {
                fact.count = count(fact.kind);
            }
        }
    }
}

impl Game {
    pub(super) fn source_counter_quantity_live(
        &self,
        source: ObjectRef,
        kind: CounterKind,
    ) -> Option<u64> {
        self.state
            .objects
            .get(source.card as usize)
            .filter(|object| object.generation == source.generation)
            .map(|_| self.counter_count(source.card, kind))
    }
    pub(super) fn refresh_local_source_counter_quantities(&self, item: &mut SpellView) {
        if item.source_counter_lki.is_some()
            && self
                .state
                .objects
                .get(item.object.card as usize)
                .is_some_and(|object| object.generation == item.object.generation)
        {
            let source = item.object;
            item.refresh_counter_quantities(|kind| {
                self.source_counter_quantity_live(source, kind)
                    .expect("exact live quantity source")
            });
        }
    }
    pub(super) fn capture_source_counter_departure(&mut self, before: &ObjectView) {
        for item in self
            .state
            .stack
            .iter_mut()
            .chain(&mut self.state.pending_triggers)
            .chain(&mut self.state.ordering_triggers)
            .chain(self.state.resolving.iter_mut().map(|r| &mut r.item))
            .chain(self.state.announcing.iter_mut().map(|a| &mut a.item))
        {
            if item.object == before.reference {
                item.refresh_counter_quantities(|kind| before.counters.get(kind));
            }
        }
    }
}

impl Game {
    pub(super) fn blocker_subtype_excluded(&self, attacker: CardId, blocker: CardId) -> bool {
        self.state.objects[attacker as usize].card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                let crate::cards::StaticAbilityFragment::CannotBeBlockedBySubtype(subtype) = *fragment else { return false; };
                self.has_subtype(blocker, subtype)
                    && self.continuous_fragment_available(attacker, |fragment| {
                        matches!(fragment, crate::cards::StaticAbilityFragment::CannotBeBlockedBySubtype(value) if *value == subtype)
                    })
            })
        })
    }
}
