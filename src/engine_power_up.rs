//! CR702.193: entry-turn mana-cost reduction and exact-ability lifetime limits.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PowerUpUseView {
    pub definition: Card,
    pub ability: u16,
    pub borrowed_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UseRow {
    object: ObjectRef,
    value: PowerUpUseView,
}
pub(super) type Rows = Option<SharedVec<UseRow>>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PowerUpCosts {
    pub reductions: Vec<ManaCost>,
}
impl PowerUpCosts {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.reductions.len() * std::mem::size_of::<ManaCost>()
    }
}
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<UseRow>>() + r.len() * std::mem::size_of::<UseRow>()
    })
}
/// Apply generic and typed reductions after X is included (CR118.7, 702.193b).
/// Excess typed reduction pays generic, but never another mana type.
pub(super) fn reduce(
    base: ManaCost,
    reduction: ManaCost,
    generic: u32,
    x: u32,
) -> Option<ManaCost> {
    let colored = std::array::from_fn(|i| base.colored[i].saturating_sub(reduction.colored[i]));
    let excess = (0..6)
        .map(|i| u64::from(reduction.colored[i].saturating_sub(base.colored[i])))
        .sum::<u64>();
    let raw = u64::from(base.generic) + u64::from(x);
    let reduction = excess + u64::from(reduction.generic) + u64::from(generic);
    Some(ManaCost {
        colored,
        generic: u32::try_from(raw.saturating_sub(reduction)).ok()?,
    })
}
/// Ordinary activations retain their allocation-free mana-options traversal.
pub(super) struct ActivationManaCosts {
    bases: crate::cards::ManaCostOptions,
    power_up: Option<Box<PowerUpCosts>>,
    base: Option<ManaCost>,
    reduction_index: usize,
    generic_discount: u32,
    x: u32,
}
impl Iterator for ActivationManaCosts {
    type Item = ManaCost;
    fn next(&mut self) -> Option<ManaCost> {
        loop {
            if self.base.is_none() {
                self.base = Some(self.bases.next()?);
                self.reduction_index = 0;
            }
            const ZERO: [ManaCost; 1] = [ManaCost {
                colored: [0; 6],
                generic: 0,
            }];
            let reductions = self
                .power_up
                .as_deref()
                .map_or(&ZERO[..], |r| r.reductions.as_slice());
            let Some(&reduction) = reductions.get(self.reduction_index) else {
                self.base = None;
                continue;
            };
            self.reduction_index += 1;
            if let Some(cost) = reduce(self.base.unwrap(), reduction, self.generic_discount, self.x)
            {
                return Some(cost);
            }
        }
    }
}
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
            || u64::from(self.power_up_count(object, definition, ability, slot))
                < self.power_up_limit(object.card)
    }
    pub(super) fn power_up_generic_discount(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
    ) -> u32 {
        if !definition.power_up_abilities().contains(&ability) || !self.is_creature(source) {
            return 0;
        }
        let controller = self.state.objects[source as usize].controller;
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                *id != source as usize
                    && o.zone == Zone::Battlefield
                    && o.controller == controller
                    && o.card.power_up_other_creature_reduction() != 0
                    && self.printed_abilities_available(*id as CardId)
            })
            .fold(0u32, |total, (_, o)| {
                total.saturating_add(o.card.power_up_other_creature_reduction())
            })
    }
    pub(super) fn power_up_costs(
        &self,
        object: ObjectRef,
        definition: Card,
        ability: u16,
    ) -> Option<Box<PowerUpCosts>> {
        if !definition.power_up_abilities().contains(&ability) {
            return None;
        }
        let source = &self.state.objects[object.card as usize];
        let reductions = if source.zone == Zone::Battlefield
            && self.object_ref(object.card) == object
            && self.entered_this_turn(object)
        {
            let mana_definition = self.copiable_recipe(object).definition;
            let (colored, generic) = mana_definition.cost();
            std::iter::once(ManaCost { colored, generic })
                .chain(
                    mana_definition
                        .power_up_mana_cost_alternatives()
                        .iter()
                        .copied(),
                )
                .collect()
        } else {
            vec![ManaCost {
                colored: [0; 6],
                generic: 0,
            }]
        };
        Some(Box::new(PowerUpCosts { reductions }))
    }
    pub(super) fn item_power_up_costs(&self, item: &SpellView) -> Option<Box<PowerUpCosts>> {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return None;
        };
        self.power_up_costs(item.object, item.source_card, ability)
    }
    pub(super) fn activation_mana_costs(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
        x: u32,
    ) -> ActivationManaCosts {
        ActivationManaCosts {
            bases: definition.activation(ability).cost.mana_options(),
            power_up: self.power_up_costs(self.object_ref(source), definition, ability),
            base: None,
            reduction_index: 0,
            generic_discount: self.activation_generic_discount(source, definition, ability),
            x,
        }
    }
    pub(super) fn record_power_up_use(&mut self, item: &SpellView, ability: u16) {
        if !item.source_card.power_up_abilities().contains(&ability)
            || !self.on_battlefield(item.card)
            || self.object_ref(item.card) != item.object
        {
            return;
        }
        let rows = self.state.power_up_uses.get_or_insert_default();
        if let Some(row) = rows.iter_mut().find(|r| {
            r.object == item.object
                && r.value.definition == item.source_card
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
                object: item.object,
                value: PowerUpUseView {
                    definition: item.source_card,
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
        if self
            .state
            .entered_this_turn
            .as_ref()
            .is_some_and(|r| r.contains(&object))
        {
            let rows = self.state.entered_this_turn.as_mut().unwrap();
            rows.retain(|r| *r != object);
            if rows.is_empty() {
                self.state.entered_this_turn = None;
            }
        }
    }
}
