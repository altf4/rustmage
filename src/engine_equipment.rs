//! Exact-incarnation attachment links and continuous Equipment bonuses.
use super::*;
impl Game {
    /// CR303.4b: attachment, irrespective of Aura controller or pending SBA,
    /// defines enchanted. Equipment and stale recipient incarnations do not.
    pub(super) fn is_enchanted(&self, id: CardId) -> bool {
        self.on_battlefield(id)
            && self.state.attachment_sources.iter().any(|&aura| {
                let object = &self.state.objects[aura as usize];
                object.zone == Zone::Battlefield
                    && ability_world::CharacteristicWorld::is_aura(self, aura)
                    && object.attached_to == Some(self.object_ref(id))
            })
    }

    pub(super) fn attached_permanent(&self, item: &SpellView) -> Option<CardId> {
        let source = item.source?;
        let target =
            if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object {
                self.state.objects[source.id as usize].attached_to
            } else {
                item.source_attachment
            }?;
        (self.on_battlefield(target.card) && self.object_ref(target.card) == target)
            .then_some(target.card)
    }
    pub(super) fn attached_subject(&self, item: &SpellView) -> Option<CardId> {
        if let Some(event) = &item.attachment_trigger {
            return (self.on_battlefield(event.recipient.card)
                && self.object_ref(event.recipient.card) == event.recipient
                && self.is_creature(event.recipient.card))
            .then_some(event.recipient.card);
        }
        self.attached_permanent(item)
            .filter(|&id| self.is_creature(id))
    }

    pub(super) fn keywords(&self, id: CardId) -> Keywords {
        self.characteristic_query().keywords(id)
    }

    pub(super) fn equipment_can_attach(&self, source: ObjectRef, target: ObjectRef) -> bool {
        can_attach(self, source, target)
    }
    pub(super) fn attach_equipment(&mut self, source: ObjectRef, target: ObjectRef) -> bool {
        if !self.equipment_can_attach(source, target) {
            return false;
        }
        if self.state.objects[source.card as usize].attached_to != Some(target) {
            self.retimestamp_live_static_source(source);
            self.retimestamp_hand_size_source(source);
            self.renew_copy_grant_origin(source, false);
        }
        self.state.objects[source.card as usize].attached_to = Some(target);
        true
    }
    pub(super) fn attach_reflexive_subjects(&mut self, item: &SpellView) -> bool {
        let Some(recipient) = item
            .targets
            .iter()
            .find(|b| b.group == 0 && self.same_target_object(**b))
            .and_then(|b| b.object)
        else {
            return false;
        };
        let Some(subject) = item.reflexive_subject.as_ref() else {
            return false;
        };
        let equipment: Vec<_> = std::iter::once(subject.object)
            .chain(subject.additional_objects.iter().copied())
            .filter(|&source| {
                self.equipment_can_attach(source, recipient)
                    && self.state.objects[source.card as usize].attached_to != Some(recipient)
            })
            .collect();
        self.order_equipment_attachments(item, recipient, equipment)
    }
    pub(super) fn attach_selected_equipment_to_attached(
        &mut self,
        item: &SpellView,
        group: u16,
    ) -> bool {
        let Some(recipient) = self.attached_subject(item).map(|id| self.object_ref(id)) else {
            return false;
        };
        // Freeze the selected exact legal objects before the controller chooses
        // their relative timestamps during resolution (CR613.7m).
        let equipment: Vec<_> = item
            .targets
            .iter()
            .filter(|b| b.group == group)
            .filter_map(|b| b.object)
            .filter(|&r| self.equipment_can_attach(r, recipient))
            .filter(|r| self.state.objects[r.card as usize].attached_to != Some(recipient))
            .collect();
        self.order_equipment_attachments(item, recipient, equipment)
    }
    pub(super) fn attach_groups_reflexive(
        &mut self,
        item: &SpellView,
        equipment: u16,
        recipient: u16,
        ability: u16,
    ) -> bool {
        let Some(recipient) = item
            .targets
            .iter()
            .find(|b| b.group == recipient && self.same_target_object(**b))
            .and_then(|b| b.object)
        else {
            return false;
        };
        let equipment = item
            .targets
            .iter()
            .filter(|b| b.group == equipment && self.same_target_object(**b))
            .filter_map(|b| b.object)
            .filter(|&r| self.equipment_can_attach(r, recipient))
            .filter(|r| self.state.objects[r.card as usize].attached_to != Some(recipient))
            .collect();
        self.order_equipment_attachments_reflexive(item, recipient, equipment, Some(ability))
    }
    pub(super) fn attach_equipment_batch(
        &mut self,
        equipment: &[ObjectRef],
        recipient: ObjectRef,
    ) -> usize {
        // Recheck the complete final set before any attachment changes. No
        // partial batch is exposed while choosing relative timestamps.
        let equipment: Vec<_> = equipment
            .iter()
            .copied()
            .filter(|&source| {
                self.equipment_can_attach(source, recipient)
                    && self.state.objects[source.card as usize].attached_to != Some(recipient)
            })
            .collect();
        for &source in &equipment {
            self.state.objects[source.card as usize].attached_to = Some(recipient);
        }
        let attached = equipment.len();
        for source in equipment {
            self.retimestamp_live_static_source(source);
            self.retimestamp_hand_size_source(source);
            self.renew_copy_grant_origin(source, false);
        }
        attached
    }
}

/// Effect attachment has no equip activation or controller restriction.
pub(super) fn can_attach<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    target: ObjectRef,
) -> bool {
    source != target
        && world.object(source.card).zone == Zone::Battlefield
        && world.reference(source.card) == source
        && world.is_equipment(source.card)
        && world.kinds(source.card) & 1 == 0
        && world.object(target.card).zone == Zone::Battlefield
        && world.reference(target.card) == target
        && world.kinds(target.card) & 1 != 0
        && !characteristic_query::CharacteristicQuery::in_world(world)
            .keywords(target.card)
            .contains(Keywords::PROTECTION_EVERYTHING)
}
