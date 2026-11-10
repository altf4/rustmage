//! Opt-in public source quantities. No history rows or hot-object fields.
use super::*;
use crate::cards::LibraryLook;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceQuantityView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_creature_type: Option<crate::cards::CreatureType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_count: Option<u64>,
    /// Exact Equipment incarnations attached to the source before departure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_equipment: Option<Box<[ObjectRef]>>,
    /// Signed source facts for an intervening creature-stat comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creature_comparison: Option<CreatureComparisonFacts>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct SelectedQuantities {
    power: bool,
    toughness: bool,
    equipment: bool,
    attached_equipment: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn selected(program: &[Instruction]) -> SelectedQuantities {
    fn amount(value: AmountValue, result: &mut SelectedQuantities) {
        match value {
            AmountValue::SourcePower => result.power = true,
            AmountValue::SourceToughness => result.toughness = true,
            AmountValue::SourceEquipmentCount => result.equipment = true,
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
    let mut result = SelectedQuantities::default();
    for instruction in program {
        match *instruction {
            Instruction::Modal(modes)
            | Instruction::ModalOnce(modes)
            | Instruction::ModalOncePerTurn(modes)
            | Instruction::ModalUpToX { modes, .. } => {
                for mode in modes {
                    let nested = selected(mode.program);
                    result.power |= nested.power;
                    result.toughness |= nested.toughness;
                    result.equipment |= nested.equipment;
                    result.attached_equipment |= nested.attached_equipment;
                }
            }
            Instruction::CounterUnlessPayCosts { costs, .. }
            | Instruction::MayPayCosts { costs, .. } => {
                for cost in costs {
                    for value in cost.extra_generic.into_iter().chain(cost.life) {
                        amount(value, &mut result);
                    }
                }
            }
            Instruction::SetBaseStatsFromSource { .. } => result.toughness = true,
            Instruction::AttachPriorSourceEquipment { .. } => result.attached_equipment = true,
            Instruction::SelectLibraryTo(effect)
            | Instruction::SelectLibraryToHandRememberCount(effect)
            | Instruction::SelectLibraryThenMayTransform(effect)
            | Instruction::SelectLibraryThenReflexiveEquipment { effect, .. } => {
                match effect.look {
                    LibraryLook::Amount(value)
                    | LibraryLook::UntilNonlandCount(value)
                    | LibraryLook::UntilCount { amount: value, .. } => amount(value, &mut result),
                    _ => (),
                }
                if let Some(value) = effect.maximum_mana_value {
                    amount(value, &mut result);
                }
            }
            Instruction::UpToTargets(value)
            | Instruction::RememberCount(value)
            | Instruction::IfAmountAtLeast { value, .. }
            | Instruction::IfAmountParity { value, .. }
            | Instruction::PutLandCards { maximum: value, .. }
            | Instruction::DestroyTargetsAtMostManaValue(value)
            | Instruction::DiscardScaled(_, value)
            | Instruction::AddCastEntryCounters { amount: value, .. }
            | Instruction::SearchScaled { maximum: value, .. }
            | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
            | Instruction::SetCreatureForm { base: value, .. }
            | Instruction::MoveCountersFromSource { amount: value, .. }
            | Instruction::RevealTargetHandSubsetDiscard(value)
            | Instruction::DrawScaled(_, value)
            | Instruction::DrawRemembered(value)
            | Instruction::MayMillAmount(value)
            | Instruction::Mill(_, MillAmount::Amount(value))
            | Instruction::GainLifeScaled(_, value)
            | Instruction::LoseLifeScaled(_, value)
            | Instruction::DrainLifeScaled(value)
            | Instruction::LoseOpponentGainSame(value)
            | Instruction::DamageScaled(value)
            | Instruction::DamageWithControllerScaled {
                controller_damage: value,
                ..
            }
            | Instruction::AddColorlessScaled(value)
            | Instruction::AddManaScaled { amount: value, .. }
            | Instruction::AddManaChosenScaled { amount: value, .. }
            | Instruction::AddCombatRedScaled(value)
            | Instruction::Mobilize { amount: value, .. }
            | Instruction::CounterUnlessPayScaled { amount: value, .. }
            | Instruction::CastHandFreeUpTo(value)
            | Instruction::PutCountersScaled { amount: value, .. }
            | Instruction::PutPermanentCountersScaled { amount: value, .. }
            | Instruction::PutLegendaryPermanentCountersScaled { amount: value, .. }
            | Instruction::GrowScaled { amount: value, .. }
            | Instruction::GrowFilteredScaled { amount: value, .. }
            | Instruction::GrowCreaturesScaled { amount: value, .. }
            | Instruction::GrowPermanentGroupScaled { amount: value, .. }
            | Instruction::CreateTokensScaled { amount: value, .. }
            | Instruction::CreateTokensWithSourceDeparture { amount: value, .. }
            | Instruction::EndureScaled { amount: value, .. }
            | Instruction::CreateSizedToken { size: value, .. }
            | Instruction::CreateTokensWithCounters {
                counters: value, ..
            }
            | Instruction::CreateTokensThenCounters {
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
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter())
                    .map(|frame| &frame.announcement.item),
            )
    }
    pub(super) fn equipped_creature_count(&self, player: Player) -> u128 {
        self.state
            .attachment_sources
            .iter()
            .filter_map(|&id| {
                let recipient = self.state.objects[id as usize].attached_to?;
                let creature = &self.state.objects[recipient.card as usize];
                (creature.controller == player
                    && ability_world::CharacteristicWorld::equipment_attached_to(
                        self, id, recipient,
                    )
                    && self.is_creature(recipient.card))
                .then_some(recipient)
            })
            .collect::<std::collections::HashSet<_>>()
            .len() as u128
    }
    pub(super) fn source_equipment_count(&self, source: ObjectRef) -> Option<u64> {
        (self.on_battlefield(source.card) && self.object_ref(source.card) == source).then(|| {
            self.source_equipment_refs(source)
                .count()
                .try_into()
                .capacity("equipment quantity exhausted")
        })
    }
    pub(super) fn source_equipment_refs(
        &self,
        source: ObjectRef,
    ) -> impl Iterator<Item = ObjectRef> + '_ {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter_map(move |(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && object.attached_to == Some(source)
                    && self.has_subtype(id, "Equipment"))
                .then(|| self.object_ref(id))
            })
    }
    pub(super) fn source_quantity_facts(
        &self,
        program: &[Instruction],
        targets: TargetPlan,
        source: &ObjectView,
    ) -> Option<Box<SourceQuantityView>> {
        let quantities = selected(program);
        let chosen_type = targets.reads_source_creature_type() || reads_chosen_type(program);
        (quantities != SelectedQuantities::default() || chosen_type).then(|| {
            Box::new(SourceQuantityView {
                power: quantities.power.then_some(source.power),
                creature_comparison: None,
                chosen_creature_type: if chosen_type {
                    match source.entry_choice {
                        Some(EntryChoiceValue::CreatureType(kind)) => Some(kind),
                        _ => None,
                    }
                } else {
                    None
                },
                toughness: quantities.toughness.then_some(if source.is_creature() {
                    source.toughness
                } else {
                    0
                }),
                equipment_count: quantities
                    .equipment
                    .then(|| self.source_equipment_count(source.reference).unwrap_or(0)),
                attached_equipment: quantities
                    .attached_equipment
                    .then(|| self.source_equipment_refs(source.reference).collect()),
            })
        })
    }
    /// Bind to the final immutable activation program, not a same-index printed
    /// ability on a borrower or acquired recipient. Only requested facts allocate.
    pub(super) fn bind_activation_source_quantities(
        &self,
        item: &mut SpellView,
        program: &[Instruction],
    ) {
        let Some(source) = item.source.map(|source| source.object) else {
            return;
        };
        if !self
            .state
            .objects
            .get(source.card as usize)
            .is_some_and(|object| object.generation == source.generation)
        {
            // A mana payment may already have sacrificed the source. Its exact
            // departure facts must survive final payment of the parent ability.
            return;
        }
        item.source_counter_lki = card_quantities::counter_quantity_facts(program, |kind| {
            self.source_counter_quantity_live(source, kind)
                .expect("exact activated quantity source")
        });
        let targets = item.program().map_or(TargetPlan::NONE, |(plan, _)| plan);
        if selected(program) != SelectedQuantities::default()
            || reads_chosen_type(program)
            || targets.reads_source_creature_type()
        {
            item.source_quantity_lki =
                self.source_quantity_facts(program, targets, &self.object_view(source.card));
        } else {
            item.source_quantity_lki = None;
        }
    }
    /// The local paying packet is outside State during its own source costs.
    /// Refresh after counter payment and before any source-departure cost.
    pub(super) fn refresh_local_source_quantities(&self, item: &mut SpellView) {
        let Some(source) = item.source.map(|source| source.object) else {
            return;
        };
        if item.source_quantity_lki.is_none()
            || !self
                .state
                .objects
                .get(source.card as usize)
                .is_some_and(|object| object.generation == source.generation)
        {
            return;
        }
        let view = self.object_view(source.card);
        let saved = item.source_quantity_lki.as_mut().unwrap();
        if saved.power.is_some() {
            saved.power = Some(view.power);
        }
        if saved.toughness.is_some() {
            saved.toughness = Some(if view.is_creature() {
                view.toughness
            } else {
                0
            });
        }
        if saved.equipment_count.is_some() {
            saved.equipment_count = Some(self.source_equipment_count(source).unwrap_or(0));
        }
        if saved.attached_equipment.is_some() {
            saved.attached_equipment = Some(self.source_equipment_refs(source).collect());
        }
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
                let mut selected = SelectedQuantities::default();
                for item in self.source_quantity_packets().filter(|item| {
                    item.source
                        .is_some_and(|identity| identity.object == source.reference)
                }) {
                    if let Some(facts) = &item.source_quantity_lki {
                        selected.power |= facts.power.is_some();
                        selected.toughness |= facts.toughness.is_some();
                        selected.equipment |= facts.equipment_count.is_some();
                        selected.attached_equipment |= facts.attached_equipment.is_some();
                    }
                }
                (selected != SelectedQuantities::default()).then(|| {
                    (
                        source.reference,
                        SourceQuantityView {
                            power: selected.power.then_some(source.power),
                            creature_comparison: None,
                            chosen_creature_type: None,
                            toughness: selected.toughness.then_some(if source.is_creature() {
                                source.toughness
                            } else {
                                0
                            }),
                            equipment_count: selected.equipment.then(|| {
                                self.source_equipment_count(source.reference).unwrap_or(0)
                            }),
                            attached_equipment: selected
                                .attached_equipment
                                .then(|| self.source_equipment_refs(source.reference).collect()),
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
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter_mut()
                    .flat_map(|frames| frames.iter_mut())
                    .map(|frame| &mut frame.announcement.item),
            )
        {
            if let Some(saved) = &mut item.source_quantity_lki
                && let Some((_, frozen)) = facts
                    .iter()
                    .find(|(object, _)| item.source.is_some_and(|source| *object == source.object))
            {
                if saved.power.is_some() {
                    saved.power = frozen.power;
                }
                if saved.toughness.is_some() {
                    saved.toughness = frozen.toughness;
                }
                if saved.equipment_count.is_some() {
                    saved.equipment_count = frozen.equipment_count;
                }
                if saved.attached_equipment.is_some() {
                    saved.attached_equipment = frozen.attached_equipment.clone();
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

fn reads_chosen_type(program: &[Instruction]) -> bool {
    fn filter(value: crate::cards::LibrarySelectionFilter) -> bool {
        use crate::cards::LibrarySelectionFilter as F;
        match value {
            F::SourceChosenCreatureType => true,
            F::AnyOf(rows) | F::AllOf(rows) => rows.iter().any(|row| filter(*row)),
            _ => false,
        }
    }
    program.iter().any(|instruction| match instruction {
        Instruction::SelectLibrary(spec) => filter(spec.filter),
        Instruction::SelectLibraryTo(effect)
        | Instruction::SelectLibraryToHandRememberCount(effect)
        | Instruction::SelectLibraryThenMayTransform(effect)
        | Instruction::SelectLibraryThenReflexiveEquipment { effect, .. } => {
            filter(effect.spec.filter)
        }
        Instruction::Modal(modes)
        | Instruction::ModalOnce(modes)
        | Instruction::ModalOncePerTurn(modes)
        | Instruction::ModalUpToX { modes, .. } => modes.iter().any(|mode| {
            mode.targets.reads_source_creature_type() || reads_chosen_type(mode.program)
        }),
        _ => false,
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A linked as-entry choice belongs to this ability acquisition, even if a later copy masks it.
    pub(super) fn source_chosen_creature_type(
        &self,
        item: &SpellView,
    ) -> Option<crate::cards::CreatureType> {
        item.source_quantity_lki
            .as_ref()
            .and_then(|facts| facts.chosen_creature_type)
    }
}
