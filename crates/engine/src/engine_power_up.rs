//! CR702.193: entry-turn mana-cost reduction and exact-ability lifetime limits.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PowerUpUseView {
    pub definition: Card,
    pub ability: u16,
    pub borrowed_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
    pub count: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UseRow {
    object: ObjectRef,
    value: PowerUpUseView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<UseRow>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ActivationCostReductions {
    /// Entry-turn Power-up mana reductions, if any.
    pub reductions: Vec<ManaCost>,
    /// The capped part of the provisional discount before the total is locked.
    pub leave_one: u64,
    /// Amount that exceeds the ordinary inline discount field.
    pub extra_ordinary: u64,
    /// Exact payable totals awaiting the player's reduction-order choice.
    pub order_costs: Option<[ManaCost; 2]>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivationCostReductions {
    pub(super) fn options(context: Option<&Self>) -> &[ManaCost] {
        context
            .filter(|costs| !costs.reductions.is_empty())
            .map_or(&[ManaCost::ZERO], |costs| costs.reductions.as_slice())
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.reductions.len() * std::mem::size_of::<ManaCost>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<UseRow>>() + r.len() * std::mem::size_of::<UseRow>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ActivationDiscounts {
    pub ordinary: u64,
    pub leave_one: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivationDiscounts {
    pub(super) fn with_ordinary(self, amount: u64) -> Self {
        Self {
            ordinary: self.ordinary.saturating_add(amount),
            ..self
        }
    }
}

/// Apply generic and typed reductions after X and increases (CR118.7, 601.2f).
/// Excess typed reduction pays generic, but never another mana type.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn reduced_totals(
    base: ManaCost,
    reduction: ManaCost,
    discounts: ActivationDiscounts,
    x_and_increases: u64,
) -> [Option<ManaCost>; 2] {
    let colored = std::array::from_fn(|i| base.colored[i].saturating_sub(reduction.colored[i]));
    let excess = (0..6)
        .map(|i| u64::from(reduction.colored[i].saturating_sub(base.colored[i])))
        .sum::<u64>();
    let Some(raw) = u64::from(base.generic).checked_add(x_and_increases) else {
        return [None; 2];
    };
    let ordinary = discounts
        .ordinary
        .saturating_add(u64::from(reduction.generic))
        .saturating_add(excess);
    // Applying capped reductions first gives the minimum. Applying ordinary
    // reductions first can leave exactly one mana if a capped effect still
    // has to reduce the remaining generic mana and all colored mana is gone.
    let capped = discounts
        .leave_one
        .min(raw.saturating_sub(u64::from(base.colored == [0; 6])));
    let uncapped = raw.saturating_sub(ordinary);
    let Ok(generic) = u32::try_from(uncapped.saturating_sub(capped)) else {
        return [None; 2];
    };
    [
        Some(ManaCost { colored, generic }),
        (generic == 0 && colored == [0; 6] && uncapped != 0).then_some(ManaCost {
            colored,
            generic: 1,
        }),
    ]
}
/// Ordinary activations retain their allocation-free mana-options traversal.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) struct ActivationManaCosts {
    bases: crate::cards::mana_symbols::ManaOptions,
    power_up: Option<Box<ActivationCostReductions>>,
    base: Option<ManaCost>,
    reduction_index: usize,
    next_total: Option<ManaCost>,
    discounts: ActivationDiscounts,
    generic_increase: u32,
    x: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Iterator for ActivationManaCosts {
    type Item = ManaCost;
    fn next(&mut self) -> Option<ManaCost> {
        if let Some(total) = self.next_total.take() {
            return Some(total);
        }
        loop {
            if self.base.is_none() {
                self.base = Some(self.bases.next()?);
                self.reduction_index = 0;
            }
            let reductions = ActivationCostReductions::options(self.power_up.as_deref());
            let Some(&reduction) = reductions.get(self.reduction_index) else {
                self.base = None;
                continue;
            };
            self.reduction_index += 1;
            let [minimum, alternate] = reduced_totals(
                self.base.unwrap(),
                reduction,
                self.discounts,
                self.x + u64::from(self.generic_increase),
            );
            if minimum.is_some() {
                self.next_total = alternate;
                return minimum;
            }
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn entered_this_turn(&self, object: ObjectRef) -> bool {
        self.state
            .entered_this_turn
            .as_ref()
            .is_some_and(|r| r.contains(&object))
    }
    pub(super) fn power_up_use_views(&self, object: ObjectRef) -> Vec<PowerUpUseView> {
        self.state
            .power_up_uses
            .as_ref()
            .into_iter()
            .flat_map(|r| r.iter())
            .filter(|r| r.object == object)
            .map(|r| r.value)
            .collect()
    }
    pub(super) fn power_up_count(
        &self,
        object: ObjectRef,
        definition: Card,
        ability: u16,
        slot: Option<u64>,
    ) -> u32 {
        self.state
            .power_up_uses
            .as_ref()
            .into_iter()
            .flat_map(|r| r.iter())
            .find(|r| {
                r.object == object
                    && r.value.definition == definition
                    && r.value.ability == ability
                    && r.value.borrowed_slot == slot
                    && r.value.origin == self.copy_ability_origin(object)
            })
            .map_or(0, |r| r.value.count)
    }
    pub(super) fn power_up_limit(&self, source: CardId) -> u64 {
        let controller = self.state.objects[source as usize].controller;
        1 + self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == controller
                    && o.card.power_up_additional_uses() != 0
                    && self.printed_abilities_available(*id as CardId)
            })
            .map(|(_, o)| u64::from(o.card.power_up_additional_uses()))
            .sum::<u64>()
    }
    pub(super) fn power_up_available(
        &self,
        object: ObjectRef,
        definition: Card,
        ability: u16,
        slot: Option<u64>,
    ) -> bool {
        !definition.power_up_abilities().contains(&ability)
            || !self.power_up_prohibited_this_turn()
                && u64::from(self.power_up_count(object, definition, ability, slot))
                    < self.power_up_limit(object.card)
    }
    pub(super) fn power_up_costs(
        &self,
        object: ObjectRef,
        definition: Card,
        ability: u16,
    ) -> Option<Box<ActivationCostReductions>> {
        ability_world::CharacteristicWorld::activation_power_up_costs(
            self, object, definition, ability,
        )
    }
    pub(super) fn capture_activation_discounts(
        &self,
        item: &SpellView,
    ) -> (u32, Option<Box<ActivationCostReductions>>) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return (0, None);
        };
        let Some(source) = item.source else {
            return (0, None);
        };
        let discounts = self.item_activation_discount(item);
        let mut context = self.power_up_costs(source.object, source.card, ability);
        if discounts.leave_one != 0 || discounts.ordinary > u64::from(u32::MAX) {
            let context = context.get_or_insert_with(Box::default);
            context.leave_one = discounts.leave_one;
            context.extra_ordinary = discounts.ordinary.saturating_sub(u64::from(u32::MAX));
        }
        let ordinary = u32::try_from(discounts.ordinary).unwrap_or(u32::MAX);
        (ordinary, context)
    }
    pub(super) fn activation_mana_costs(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
        x: u32,
    ) -> ActivationManaCosts {
        ActivationManaCosts {
            bases: definition.activation_cost_options(ability),
            power_up: self.power_up_costs(self.object_ref(source), definition, ability),
            base: None,
            reduction_index: 0,
            next_total: None,
            // A target is still to be chosen. Keep every raw choice that might
            // fit after its color reduction; target-aware funding recomputes
            // the exact total from the raw choice before accepting any target.
            discounts: self
                .activation_generic_discount(source, definition, ability)
                .with_ordinary(5 * u64::from(definition.activation_target_color_discount(ability))),
            generic_increase: ability_world::CharacteristicWorld::named_activation_generic_increase(
                self, source,
            ),
            x: u64::from(x.max(definition.activation(ability).minimum_x()))
                * u64::from(definition.activation_x_symbols(ability)),
        }
    }
    pub(super) fn record_power_up_use(&mut self, item: &SpellView, ability: u16) {
        let Some(source) = item.source else {
            return;
        };
        if !source.card.power_up_abilities().contains(&ability)
            || !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
        {
            return;
        }
        let rows = self.state.power_up_uses.get_or_insert_default();
        if let Some(row) = rows.iter_mut().find(|r| {
            r.object == source.object
                && r.value.definition == source.card
                && r.value.ability == ability
                && r.value.borrowed_slot == item.borrowed_slot
                && r.value.origin == item.source_ability_origin
        }) {
            row.value.count = row
                .value
                .count
                .checked_add(1)
                .capacity("Power-up use count exhausted");
        } else {
            rows.push(UseRow {
                object: source.object,
                value: PowerUpUseView {
                    definition: source.card,
                    ability,
                    borrowed_slot: item.borrowed_slot,
                    origin: item.source_ability_origin,
                    count: 1,
                },
            });
        }
    }
    pub(super) fn forget_power_up_object(&mut self, object: ObjectRef) {
        if self
            .state
            .power_up_uses
            .as_ref()
            .is_some_and(|r| r.iter().any(|r| r.object == object))
        {
            let rows = self.state.power_up_uses.as_mut().unwrap();
            rows.retain(|r| r.object != object);
            if rows.is_empty() {
                self.state.power_up_uses = None;
            }
        }
        // Exact entry history also qualifies damage from a departed permanent's
        // last known incarnation. Retain it until the ordinary turn reset;
        // another incarnation never matches this ObjectRef.
    }
}
