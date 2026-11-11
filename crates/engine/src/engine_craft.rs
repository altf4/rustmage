//! Ordered craft material groups and exact post-return exile receipts.
use super::*;
use crate::cards::CraftMaterials;

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
        self.state
            .linked_choices
            .as_ref()
            .into_iter()
            .flat_map(|state| &state.craft_links)
            .filter_map(|link| {
                let object = &self.state.objects[link.exiled.card as usize];
                (link.source == source
                    && self.object_ref(link.exiled.card) == link.exiled
                    && object.zone == Zone::Exile
                    && object.is_card())
                .then_some(link.exiled)
            })
            .collect()
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
                });
        }
    }
}
