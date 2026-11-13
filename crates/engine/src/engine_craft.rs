//! Ordered craft material groups and exact post-return exile receipts.
use super::*;
use crate::cards::CraftMaterials;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn material_matches<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    kind: EffectChoiceKind,
    object: ObjectRef,
) -> bool {
    let Some(CraftMaterials::DistinctSubtypes(slots)) = kind.craft_materials() else {
        return true;
    };
    let subtypes = world.subtypes(object.card);
    slots
        .iter()
        .any(|subtype| subtypes.has(world.definition(object.card), subtype))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn material_group_matches<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    materials: CraftMaterials,
    selected: &[ObjectRef],
) -> bool {
    if materials
        .maximum()
        .is_some_and(|maximum| selected.len() > maximum)
        || selected
            .iter()
            .enumerate()
            .any(|(index, object)| selected[..index].contains(object))
    {
        return false;
    }
    let CraftMaterials::DistinctSubtypes(slots) = materials else {
        return true;
    };
    assert!(
        slots.len() <= u64::BITS as usize,
        "craft subtype slots exhausted"
    );
    let choices = selected
        .iter()
        .map(|object| {
            let subtypes = world.subtypes(object.card);
            slots
                .iter()
                .enumerate()
                .fold(0u64, |mask, (index, subtype)| {
                    mask | (u64::from(subtypes.has(world.definition(object.card), subtype))
                        << index)
                })
        })
        .collect::<Vec<_>>();
    fn assign(choices: &[u64], used: u64) -> bool {
        let Some((&mask, later)) = choices.split_first() else {
            return true;
        };
        let mut available = mask & !used;
        while available != 0 {
            let slot = 1u64 << available.trailing_zeros();
            if assign(later, used | slot) {
                return true;
            }
            available &= !slot;
        }
        false
    }
    assign(&choices, 0)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn active_craft_payment_actor(&self) -> Option<Player> {
        self.state
            .announcing
            .iter()
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter().rev())
                    .map(|frame| &frame.announcement),
            )
            .find_map(|announcement| {
                announcement
                    .ordered_craft()
                    .then_some(announcement.item.controller)
            })
    }

    pub(super) fn craft_colors<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        source: ObjectRef,
    ) -> u8 {
        self.craft_materials_in_world(world, source)
            .fold(0, |mask, object| mask | world.colors(object.card))
            & 0b1_1111
    }

    pub(super) fn craft_material_power<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        source: ObjectRef,
    ) -> i128 {
        // Craft links only arise on entry. An exiled card cannot itself be
        // that same crafted incarnation, so its CDA cannot follow more links.
        if world.object(source.card).zone != Zone::Battlefield
            || world.reference(source.card) != source
        {
            return 0;
        }
        self.craft_materials_in_world(world, source)
            .map(|material| self.craft_exile_card_power(world, material))
            .try_fold(0, i128::checked_add)
            .capacity("craft material power exhausted")
    }

    fn craft_exile_card_power<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        material: ObjectRef,
    ) -> i128 {
        let object = world.object(material.card);
        let card = world.definition(material.card);
        if let Some(power) = card.dynamic_stats().and_then(|stats| stats.power)
            && world.printed_abilities_available(material.card)
        {
            return self
                .stat_value_in_world(world, power.value, material, object.owner)
                .checked_mul(power.multiplier)
                .and_then(|value| value.checked_add(power.offset))
                .capacity("craft material power exhausted");
        }
        // CR 208.3 preserves printed power on a noncreature card in exile.
        if let Some((power, _)) = card.vehicle_stats() {
            return power;
        }
        match card.effect() {
            Effect::Creature(power, _) => i128::from(power),
            _ => 0,
        }
    }

    pub(super) fn finish_craft_materials(&mut self) {
        assert!(
            self.craft_material_selection_complete(),
            "incomplete craft materials"
        );
        let a = self.state.announcing.as_mut().unwrap();
        a.additional_group.get_or_insert_default();
        a.additional_group_complete = true;
        a.additional = a
            .additional_group
            .as_ref()
            .and_then(|group| group.last().copied());
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.resume_cast_cost_selection();
    }

    pub(super) fn crafted_exiled_cards(&self, source: ObjectRef) -> Vec<ObjectRef> {
        self.craft_materials_in_world(self, source).collect()
    }

    fn craft_materials_in_world<'a, W: CharacteristicWorld + ?Sized>(
        &'a self,
        world: &'a W,
        source: ObjectRef,
    ) -> impl Iterator<Item = ObjectRef> + 'a {
        self.state
            .linked_choices
            .as_ref()
            .into_iter()
            .flat_map(|state| &state.craft_links)
            .filter_map(move |link| (link.source == source).then_some(link.exiled))
            .chain(world.projected_craft_materials(source).iter().copied())
            .filter(move |&material| {
                let object = world.object(material.card);
                object.zone == Zone::Exile
                    && object.is_card()
                    && world.reference(material.card) == material
            })
    }

    pub(super) fn crafted_exiled_creature(&self, source: ObjectRef, object: ObjectRef) -> bool {
        self.state.objects[object.card as usize].zone == Zone::Exile
            && self
                .craft_materials_in_world(self, source)
                .any(|card| card == object)
            && self.is_creature(object.card)
    }

    pub(super) fn register_craft_materials(&mut self, source: ObjectRef, materials: &[ObjectRef]) {
        for &exiled in materials {
            if self.object_ref(exiled.card) != exiled
                || self.state.objects[exiled.card as usize].zone != Zone::Exile
                || !self.state.objects[exiled.card as usize].is_card()
            {
                continue;
            }
            self.state
                .linked_choices
                .get_or_insert_default()
                .craft_links
                .push(LinkedExileView {
                    exiled_turn: self.state.turn,
                    source,
                    exiled,
                    ability_origin: None,
                    acquired_link_cohort: None,
                });
            let first_slot = graveyard_borrowing::next_slot(self, source);
            self.state
                .borrowed_links
                .get_or_insert_default()
                .push(borrowed_activations::Link {
                    source,
                    origin: None,
                    exiled,
                    first_slot,
                    width: borrowed_activations::DONOR_SLOT_WIDTH,
                    once_per_turn: true,
                    shared: false,
                    craft: true,
                    chosen_cohort: None,
                });
        }
    }
}
