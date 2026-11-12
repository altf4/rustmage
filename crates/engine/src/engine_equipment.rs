//! Exact-incarnation attachment links and continuous Equipment bonuses.
use super::*;
use crate::cards::TriggeredAbility;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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

    pub(super) fn attached_to_controlled_creature(
        &self,
        object: ObjectRef,
        player: Player,
    ) -> bool {
        self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && self.state.objects[object.card as usize]
                .attached_to
                .is_some_and(|recipient| {
                    self.on_battlefield(recipient.card)
                        && self.object_ref(recipient.card) == recipient
                        && self.state.objects[recipient.card as usize].controller == player
                        && self.is_creature(recipient.card)
                })
    }
    pub(super) fn equipment_attached_to_target(
        &self,
        item: &SpellView,
        group: u16,
        equipment: ObjectRef,
    ) -> bool {
        if !self.on_battlefield(equipment.card)
            || self.object_ref(equipment.card) != equipment
            || !ability_world::CharacteristicWorld::is_equipment(self, equipment.card)
        {
            return false;
        }
        let Some(recipient) = self.state.objects[equipment.card as usize].attached_to else {
            return false;
        };
        self.on_battlefield(recipient.card)
            && item.targets.iter().any(|bound| {
                bound.group == group
                    && bound.object == Some(recipient)
                    && self.same_target_object(*bound)
            })
    }
    pub(super) fn unattach_event_object(&mut self, item: &SpellView) {
        if let Some(event) = &item.event_object
            && self.unattach(event.object)
        {
            self.refresh_controllers();
        }
    }
    pub(super) fn unattach(&mut self, object: ObjectRef) -> bool {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return false;
        }
        let attachment = &mut self.state.objects[object.card as usize];
        if !attachment.is_attached() {
            return false;
        }
        attachment.attached_to = None;
        attachment.attached_to_player = None;
        self.forget_control_attachment(object);
        self.forget_attachment_origin(object);
        self.renew_copy_grant_origin(object, false);
        self.prune_source_bound_copies();
        true
    }
    pub(super) fn equipment_can_attach(&self, source: ObjectRef, target: ObjectRef) -> bool {
        can_attach(self, source, target)
    }
    pub(super) fn attach_equipment(&mut self, source: ObjectRef, target: ObjectRef) -> bool {
        if !self.equipment_can_attach(source, target) {
            return false;
        }
        if self.state.objects[source.card as usize].attached_to != Some(target) {
            self.forget_control_attachment(source);
            self.forget_attachment_origin(source);
            self.retimestamp_live_static_source(source);
            self.retimestamp_hand_size_source(source);
            self.renew_copy_grant_origin(source, false);
            let object = &mut self.state.objects[source.card as usize];
            object.attached_to = Some(target);
            object.attached_to_player = None;
            self.prune_source_bound_copies();
            self.self_attachment_triggers(source, target);
        }
        true
    }

    pub(super) fn queue_attachment_trigger(
        &mut self,
        observer: &ObjectView,
        index: usize,
        attachment: &ObjectView,
        recipient: ObjectRef,
    ) {
        let roles = self.state.roles.get_or_insert_default();
        let origin = roles
            .attachment_origin(attachment.reference)
            .unwrap_or_else(|| {
                let origin = roles.next_acquisition();
                roles
                    .attachment_origins
                    .push((attachment.reference, origin));
                origin
            });
        let event = AttachmentTriggerView {
            grantor: attachment.reference,
            granting_card: attachment.card,
            recipient,
            defender: None,
            granted: false,
            origin: Some(origin),
        };
        if self.queue_trigger_from(observer, index, Some(Box::new(event))) {
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .refer_objects([attachment.reference, recipient]);
        }
    }

    pub(super) fn forget_attachment_origin(&mut self, source: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles
                .attachment_origins
                .retain(|(object, _)| *object != source);
        }
    }

    pub(super) fn exact_attachment_event_is_live(&self, event: &AttachmentTriggerView) -> bool {
        event.origin.is_some()
            && self.attachment_is_live(event.grantor, event.recipient)
            && self
                .state
                .roles
                .as_ref()
                .and_then(|roles| roles.attachment_origin(event.grantor))
                == event.origin
    }

    pub(super) fn self_attachment_triggers(&mut self, attachment: ObjectRef, recipient: ObjectRef) {
        let card = self.state.objects[attachment.card as usize].card;
        let observes = |ability: &TriggeredAbility| {
            ability.condition.event() == TriggerCondition::SelfBecomesAttachedToCreature
        };
        if !card.triggers().iter().any(observes)
            || !self.attachment_is_live(attachment, recipient)
            || !self.is_creature(recipient.card)
        {
            return;
        }
        let source = self.object_view(attachment.card);
        for (index, _) in card
            .triggers()
            .iter()
            .enumerate()
            .filter(|(_, ability)| observes(ability))
        {
            self.queue_attachment_trigger(&source, index, &source, recipient);
        }
    }

    pub(super) fn new_source_attachment_pair(
        &self,
        item: &SpellView,
        group: u16,
    ) -> Option<(ObjectRef, ObjectRef)> {
        let source = item.source?.object;
        let recipient = item
            .targets
            .iter()
            .find(|bound| bound.group == group)
            .and_then(|bound| bound.object)?;
        (self.equipment_can_attach(source, recipient)
            && self.state.objects[source.card as usize].attached_to != Some(recipient))
        .then_some((source, recipient))
    }
    pub(super) fn complete_optional_source_attachment(&mut self, skip_to: usize) {
        let resolution = self.state.resolving.as_ref().unwrap();
        let Some(Instruction::MayAttachSource { recipient, .. }) =
            resolution.program.get(resolution.pc.saturating_sub(1))
        else {
            return;
        };
        let pair = self.new_source_attachment_pair(&resolution.item, *recipient);
        if !pair.is_some_and(|(source, recipient)| self.attach_equipment(source, recipient)) {
            self.state.resolving.as_mut().unwrap().pc = skip_to;
        }
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
        // Freeze the exact legal objects before their controllers choose
        // relative timestamps during resolution (CR 613.7m).
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
    #[cfg(feature = "test-support")]
    pub(super) fn attach_equipment_batch(
        &mut self,
        equipment: &[ObjectRef],
        recipient: ObjectRef,
    ) -> usize {
        self.attach_equipment_batch_objects(equipment, recipient)
            .len()
    }
    pub(super) fn attach_equipment_batch_objects(
        &mut self,
        equipment: &[ObjectRef],
        recipient: ObjectRef,
    ) -> Vec<ObjectRef> {
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
            self.forget_control_attachment(source);
            self.forget_attachment_origin(source);
            self.state.objects[source.card as usize].attached_to = Some(recipient);
        }
        for &source in &equipment {
            self.retimestamp_live_static_source(source);
            self.retimestamp_hand_size_source(source);
            self.renew_copy_grant_origin(source, false);
        }
        self.prune_source_bound_copies();
        for &source in &equipment {
            self.self_attachment_triggers(source, recipient);
        }
        equipment
    }
}

/// Effect attachment has no equip activation or controller restriction.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn can_attach<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    target: ObjectRef,
) -> bool {
    world.object(source.card).zone == Zone::Battlefield
        && can_attach_as(world, source, target, world.object(source.card).controller)
}

/// An exact existing or prospective Equipment controlled by this player.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn can_attach_as<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    target: ObjectRef,
    controller: Player,
) -> bool {
    source != target
        && world.reference(source.card) == source
        && world.is_equipment(source.card)
        && world.kinds(source.card) & 1 == 0
        && world.object(target.card).zone == Zone::Battlefield
        && world.reference(target.card) == target
        && world.kinds(target.card) & 1 != 0
        && !characteristic_query::CharacteristicQuery::in_world(world)
            .keywords(target.card)
            .protects_from_controller(
                world.colors(source.card),
                world.kinds(source.card),
                controller != world.object(target.card).controller,
            )
        && !world.protected_from_subtype(target.card, |subtype| {
            world
                .subtypes(source.card)
                .has(world.definition(source.card), subtype)
        })
}
