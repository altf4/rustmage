//! Opt-in public source quantities. No history rows or hot-object fields.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceQuantityView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_count: Option<u64>,
    /// Signed source facts for an intervening creature-stat comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creature_comparison: Option<CreatureComparisonFacts>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn selected(program: &[Instruction]) -> (bool, bool) {
    fn amount(value: AmountValue, result: &mut (bool, bool)) {
        match value {
            AmountValue::SourceToughness => result.0 = true,
            AmountValue::SourceEquipmentCount => result.1 = true,
            AmountValue::Multiple { value, .. } | AmountValue::Divided { value, .. } => {
                amount(*value, result)
            }
            AmountValue::Sum(values) => {
                for value in values {
                    amount(*value, result);
                }
            }
            _ => (),
        }
    }
    let mut result = (false, false);
    for instruction in program {
        match *instruction {
            Instruction::Modal(modes)
            | Instruction::ModalOnce(modes)
            | Instruction::ModalOncePerTurn(modes)
            | Instruction::ModalUpToX { modes, .. } => {
                for mode in modes {
                    let nested = selected(mode.program);
                    result.0 |= nested.0;
                    result.1 |= nested.1;
                }
            }
            Instruction::SetBaseStatsFromSource { .. } => result.0 = true,
            Instruction::SelectLibraryTo(effect)
            | Instruction::SelectLibraryThenMayTransform(effect) => {
                if let Some(value) = effect.maximum_mana_value {
                    amount(value, &mut result);
                }
            }
            Instruction::UpToTargets(value)
            | Instruction::IfAmountAtLeast { value, .. }
            | Instruction::PutLandCards { maximum: value, .. }
            | Instruction::DiscardScaled(_, value)
            | Instruction::AddCastEntryCounters { amount: value, .. }
            | Instruction::SearchScaled { maximum: value, .. }
            | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
            | Instruction::SetCreatureForm { base: value, .. }
            | Instruction::MoveCountersFromSource { amount: value, .. }
            | Instruction::RevealTargetHandSubsetDiscard(value)
            | Instruction::DrawScaled(_, value)
            | Instruction::GainLifeScaled(_, value)
            | Instruction::LoseLifeScaled(_, value)
            | Instruction::DrainLifeScaled(value)
            | Instruction::LoseOpponentGainSame(value)
            | Instruction::DamageScaled(value)
            | Instruction::AddColorlessScaled(value)
            | Instruction::AddManaScaled { amount: value, .. }
            | Instruction::AddCombatRedScaled(value)
            | Instruction::Mobilize { amount: value, .. }
            | Instruction::CounterUnlessPayScaled { amount: value, .. }
            | Instruction::PutCountersScaled { amount: value, .. }
            | Instruction::PutPermanentCountersScaled { amount: value, .. }
            | Instruction::GrowScaled { amount: value, .. }
            | Instruction::GrowCreaturesScaled { amount: value, .. }
            | Instruction::GrowPermanentGroupScaled { amount: value, .. }
            | Instruction::CreateTokensScaled { amount: value, .. }
            | Instruction::CreateTokensWithCounters {
                counters: value, ..
            } => amount(value, &mut result),
            _ => (),
        }
    }
    result
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn source_quantity_packets(&self) -> impl Iterator<Item = &SpellView> {
        self.state
            .stack
            .iter()
            .chain(&self.state.pending_triggers)
            .chain(&self.state.ordering_triggers)
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .chain(self.state.announcing.iter().map(|a| &a.item))
            .chain(self.state.activation_payment.iter().map(|p| &p.item))
    }
    pub(super) fn source_equipment_count(&self, source: ObjectRef) -> Option<u64> {
        (self.on_battlefield(source.card) && self.object_ref(source.card) == source).then(|| {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.attached_to == Some(source)
                        && self.has_subtype(*id as CardId, "Equipment")
                })
                .count()
                .try_into()
                .capacity("equipment quantity exhausted")
        })
    }
    pub(super) fn source_quantity_facts(
        &self,
        program: &[Instruction],
        source: &ObjectView,
    ) -> Option<Box<SourceQuantityView>> {
        let (toughness, equipment) = selected(program);
        (toughness || equipment).then(|| {
            Box::new(SourceQuantityView {
                creature_comparison: None,
                toughness: toughness.then_some(if source.is_creature() {
                    source.toughness
                } else {
                    0
                }),
                equipment_count: equipment
                    .then(|| self.source_equipment_count(source.reference).unwrap_or(0)),
            })
        })
    }
    /// Freeze before ANY member commits. Live queries remain authoritative until
    /// the source actually leaves; a failed/skipped move cannot bind a new object.
    pub(super) fn capture_source_quantities<'a>(
        &mut self,
        before: impl IntoIterator<Item = &'a ObjectView>,
    ) {
        if !self
            .source_quantity_packets()
            .any(|item| item.source_quantity_lki.is_some())
        {
            return;
        }
        let facts: Vec<_> = before
            .into_iter()
            .filter_map(|source| {
                if source.zone != Zone::Battlefield {
                    return None;
                }
                let mut selected = (false, false);
                for item in self.source_quantity_packets().filter(|item| {
                    item.source
                        .is_some_and(|identity| identity.object == source.reference)
                }) {
                    if let Some(facts) = &item.source_quantity_lki {
                        selected.0 |= facts.toughness.is_some();
                        selected.1 |= facts.equipment_count.is_some();
                    }
                }
                (selected.0 || selected.1).then(|| {
                    (
                        source.reference,
                        SourceQuantityView {
                            creature_comparison: None,
                            toughness: selected.0.then_some(if source.is_creature() {
                                source.toughness
                            } else {
                                0
                            }),
                            equipment_count: selected.1.then(|| {
                                self.source_equipment_count(source.reference).unwrap_or(0)
                            }),
                        },
                    )
                })
            })
            .collect();
        if facts.is_empty() {
            return;
        }
        for item in self
            .state
            .stack
            .iter_mut()
            .chain(&mut self.state.pending_triggers)
            .chain(&mut self.state.ordering_triggers)
            .chain(self.state.resolving.iter_mut().map(|r| &mut r.item))
            .chain(self.state.announcing.iter_mut().map(|a| &mut a.item))
            .chain(
                self.state
                    .activation_payment
                    .iter_mut()
                    .map(|p| &mut p.item),
            )
        {
            if let Some(saved) = &mut item.source_quantity_lki
                && let Some((_, frozen)) = facts
                    .iter()
                    .find(|(object, _)| item.source.is_some_and(|source| *object == source.object))
            {
                if saved.toughness.is_some() {
                    saved.toughness = frozen.toughness;
                }
                if saved.equipment_count.is_some() {
                    saved.equipment_count = frozen.equipment_count;
                }
            }
        }
    }
    pub(super) fn source_toughness(&self, item: &SpellView) -> u128 {
        let Some(source) = item.source else {
            return 0;
        };
        let value = if self.on_battlefield(source.object.card)
            && self.object_ref(source.object.card) == source.object
        {
            if self.is_creature(source.object.card) {
                self.stats(source.object.card).toughness
            } else {
                0
            }
        } else {
            item.source_quantity_lki
                .as_ref()
                .and_then(|f| f.toughness)
                .unwrap_or(0)
        };
        // An ordinary calculated +X/+X bonus treats a negative X as zero (CR107.1b).
        value.max(0) as u128
    }
    pub(super) fn source_equipment_quantity(&self, item: &SpellView) -> u128 {
        let Some(source) = item.source else {
            return 0;
        };
        u128::from(
            self.source_equipment_count(source.object)
                .unwrap_or_else(|| {
                    item.source_quantity_lki
                        .as_ref()
                        .and_then(|f| f.equipment_count)
                        .unwrap_or(0)
                }),
        )
    }
    pub(super) fn source_quantity_guard(&self, item: &SpellView) -> bool {
        let StackKind::TriggeredAbility { ability } = item.kind else {
            return true;
        };
        item.trigger_definition().triggers()[usize::from(ability)]
            .condition
            .event()
            != TriggerCondition::SelfAttacksIfEquipped
            || self.source_equipment_quantity(item) > 0
    }
}
