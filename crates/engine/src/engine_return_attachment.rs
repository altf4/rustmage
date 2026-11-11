//! Exact successful return results and public, nontargeting Equipment choices.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum ControlledEquipmentSelection {
    One(Option<&'static str>),
    Any,
    UntilNextEndStep(u16),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttachmentCallback {
    Reflexive(u16),
    UnattachAtNextEndStep(u16),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReturnAttachment {
    pub origin: ObjectRef,
    pub player: Player,
    pub recipient: Option<ObjectRef>,
    pub entering: bool,
    order: Option<Box<EquipmentOrder>>,
    callback: Option<AttachmentCallback>,
    exact_equipment: Option<Box<[ObjectRef]>>,
    required: bool,
    selected: Option<Vec<ObjectRef>>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct EquipmentOrder {
    remaining: [Vec<ObjectRef>; 2],
    chosen: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ReturnAttachment {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.exact_equipment.as_ref().map_or(0, |equipment| {
                equipment.len() * std::mem::size_of::<ObjectRef>()
            })
            + self.selected.as_ref().map_or(0, |equipment| {
                equipment.len() * std::mem::size_of::<ObjectRef>()
            })
            + self.order.as_ref().map_or(0, |order| {
                std::mem::size_of::<EquipmentOrder>()
                    + (order.remaining.iter().map(Vec::len).sum::<usize>() + order.chosen.len())
                        * std::mem::size_of::<ObjectRef>()
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct EquipmentAttachmentView {
    pub player: Player,
    pub recipient: ObjectView,
    pub candidates: Vec<ObjectRef>,
    /// Whether the effect requires selecting one legal Equipment.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    /// The current public selection when the effect allows any number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<Vec<ObjectRef>>,
    /// Mandatory relative timestamp choice for an already selected batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordering: Option<bool>,
    /// The public prefix of the simultaneous attachment timestamps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prior_public_order: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn may_attach_controlled_equipment(
        &mut self,
        item: &SpellView,
        group: u16,
        selection: ControlledEquipmentSelection,
    ) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        let recipient = item
            .targets
            .iter()
            .find(|b| b.group == group && self.same_target_object(**b))
            .and_then(|b| b.object)
            .filter(|r| {
                self.on_battlefield(r.card)
                    && self.is_creature(r.card)
                    && match selection {
                        ControlledEquipmentSelection::One(Some(subtype)) => {
                            self.has_subtype(r.card, subtype)
                        }
                        _ => true,
                    }
            });
        let Some(recipient) = recipient else {
            return false;
        };
        self.state.resolving.as_mut().unwrap().return_attachment =
            Some(Box::new(ReturnAttachment {
                origin: source.object,
                player: item.controller,
                recipient: Some(recipient),
                entering: false,
                order: None,
                callback: match selection {
                    ControlledEquipmentSelection::UntilNextEndStep(ability) => {
                        Some(AttachmentCallback::UnattachAtNextEndStep(ability))
                    }
                    _ => None,
                },
                exact_equipment: None,
                required: false,
                selected: matches!(selection, ControlledEquipmentSelection::Any).then(Vec::new),
            }));
        self.advance_return_attachment()
    }
    pub(super) fn attach_prior_source_equipment(&mut self, item: &SpellView, group: u16) -> bool {
        let Some(source) = item.source.map(|source| source.object) else {
            return false;
        };
        let Some(recipient) = item
            .targets
            .iter()
            .find(|bound| bound.group == group && self.same_target_object(**bound))
            .and_then(|bound| bound.object)
        else {
            return false;
        };
        let mut equipment: Vec<_> =
            if self.on_battlefield(source.card) && self.object_ref(source.card) == source {
                self.source_equipment_refs(source).collect()
            } else {
                item.source_quantity_lki
                    .as_ref()
                    .and_then(|facts| facts.attached_equipment.as_deref())
                    .unwrap_or(&[])
                    .to_vec()
            };
        equipment.retain(|&equipment| self.equipment_can_attach(equipment, recipient));
        self.choose_exact_equipment(item, recipient, equipment, true)
    }

    pub(super) fn reflexive_equipment_for<'a>(
        &'a self,
        item: &'a SpellView,
        recipient: ObjectRef,
    ) -> impl Iterator<Item = ObjectRef> + 'a {
        item.reflexive_subject
            .iter()
            .flat_map(|subject| {
                std::iter::once(subject.object).chain(subject.additional_objects.iter().copied())
            })
            .filter(move |&equipment| self.equipment_can_attach(equipment, recipient))
    }

    pub(super) fn choose_reflexive_equipment(
        &mut self,
        item: &SpellView,
        recipient: ObjectRef,
    ) -> bool {
        let equipment = self.reflexive_equipment_for(item, recipient).collect();
        self.choose_exact_equipment(item, recipient, equipment, false)
    }

    fn choose_exact_equipment(
        &mut self,
        item: &SpellView,
        recipient: ObjectRef,
        equipment: Vec<ObjectRef>,
        required: bool,
    ) -> bool {
        match equipment.as_slice() {
            [] => return false,
            [only] => {
                self.attach_equipment(*only, recipient);
                return false;
            }
            _ => (),
        }
        let Some(source) = item.source else {
            return false;
        };
        self.state.resolving.as_mut().unwrap().return_attachment =
            Some(Box::new(ReturnAttachment {
                origin: source.object,
                player: item.controller,
                recipient: Some(recipient),
                entering: false,
                order: None,
                callback: None,
                exact_equipment: Some(equipment.into_boxed_slice()),
                required,
                selected: None,
            }));
        self.advance_return_attachment()
    }
    pub(super) fn order_equipment_attachments(
        &mut self,
        item: &SpellView,
        recipient: ObjectRef,
        equipment: Vec<ObjectRef>,
    ) -> bool {
        self.order_equipment_attachments_reflexive(item, recipient, equipment, None)
    }
    pub(super) fn order_equipment_attachments_reflexive(
        &mut self,
        item: &SpellView,
        recipient: ObjectRef,
        equipment: Vec<ObjectRef>,
        reflexive: Option<u16>,
    ) -> bool {
        if equipment.len() < 2 {
            let attached = self.attach_equipment_batch_objects(&equipment, recipient);
            if !attached.is_empty()
                && let Some(ability) = reflexive
            {
                self.queue_reflexive_objects(
                    item,
                    ability,
                    std::iter::once(recipient).chain(attached),
                );
            }
            return false;
        }
        let Some(source) = item.source else {
            return false;
        };
        let mut remaining = [Vec::new(), Vec::new()];
        for reference in equipment {
            let controller = self.state.objects[reference.card as usize].controller;
            remaining[controller.index()].push(reference);
        }
        for candidates in &mut remaining {
            candidates.sort_unstable();
        }
        self.state.resolving.as_mut().unwrap().return_attachment =
            Some(Box::new(ReturnAttachment {
                origin: source.object,
                player: item.controller,
                recipient: Some(recipient),
                entering: false,
                callback: reflexive.map(AttachmentCallback::Reflexive),
                exact_equipment: None,
                required: false,
                selected: None,
                order: Some(Box::new(EquipmentOrder {
                    remaining,
                    chosen: Vec::new(),
                })),
            }));
        self.advance_return_attachment()
    }
    pub(super) fn return_source_with_equipment(&mut self, item: &SpellView, counter: CounterKind) {
        let Some(source) = item.source else {
            return;
        };
        if self.object_ref(source.id) != source.object
            || self.state.objects[source.id as usize].zone != Zone::Graveyard
        {
            return;
        }
        self.state.resolving.as_mut().unwrap().return_attachment =
            Some(Box::new(ReturnAttachment {
                origin: source.object,
                player: item.controller,
                recipient: None,
                entering: true,
                order: None,
                callback: None,
                exact_equipment: None,
                required: false,
                selected: None,
            }));
        let mut movement = self.entry_move(source.id, Zone::Battlefield, false);
        movement.controller = item.controller;
        movement.effect_counter = Some((counter, 1));
        movement.equipment_after = true;
        self.stage_entries(vec![movement], None);
    }
    pub(super) fn capture_return_attachment_entry(
        &mut self,
        origin: ObjectRef,
        events: &[ZoneChange],
    ) {
        let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .and_then(|r| r.return_attachment.as_mut())
        else {
            return;
        };
        if frame.entering && frame.origin == origin {
            frame.entering = false;
            frame.recipient = events
                .iter()
                .find(|event| {
                    event.before.reference == origin
                        && event.moved
                        && event.destination == Zone::Battlefield
                })
                .map(|event| event.after);
        }
    }
    fn visit_attachment_candidates(
        &self,
        frame: &ReturnAttachment,
        mut emit: impl FnMut(ObjectRef) -> bool,
    ) -> bool {
        let Some(recipient) = frame.recipient else {
            return false;
        };
        if !self.on_battlefield(recipient.card)
            || self.object_ref(recipient.card) != recipient
            || !self.is_creature(recipient.card)
        {
            return false;
        }
        if let Some(equipment) = frame.exact_equipment.as_deref() {
            return equipment.iter().copied().any(|equipment| {
                self.equipment_can_attach(equipment, recipient) && emit(equipment)
            });
        }
        self.state.objects.iter().enumerate().any(|(id, object)| {
            let equipment = self.object_ref(id as CardId);
            object.controller == frame.player
                && (!matches!(
                    frame.callback,
                    Some(AttachmentCallback::UnattachAtNextEndStep(_))
                ) || object.attached_to != Some(recipient))
                && frame
                    .selected
                    .as_ref()
                    .is_none_or(|selected| !selected.contains(&equipment))
                && self.equipment_can_attach(equipment, recipient)
                && emit(equipment)
        })
    }

    pub(super) fn advance_return_attachment(&mut self) -> bool {
        let Some(frame) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.return_attachment.as_ref())
        else {
            return false;
        };
        if frame.entering {
            return true;
        }
        if frame.order.is_some() {
            return self.advance_equipment_order();
        }
        if frame
            .selected
            .as_ref()
            .is_some_and(|selected| !selected.is_empty())
            || self.visit_attachment_candidates(frame, |_| true)
        {
            self.state.pending = Pending::EquipmentAttachment {
                player: frame.player,
            };
            return true;
        }
        self.state.resolving.as_mut().unwrap().return_attachment = None;
        false
    }
    fn advance_equipment_order(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let order = resolution
            .return_attachment
            .as_mut()
            .unwrap()
            .order
            .as_mut()
            .unwrap();
        // CR 613.7m: each controller orders their own attachments, in APNAP order.
        for player in [self.state.active, self.state.active.other()] {
            let remaining = &mut order.remaining[player.index()];
            if remaining.len() > 1 {
                self.state.pending = Pending::EquipmentAttachment { player };
                return true;
            }
            order.chosen.append(remaining);
        }
        let frame = resolution.return_attachment.take().unwrap();
        let recipient = frame.recipient.unwrap();
        let attached = self.attach_equipment_batch_objects(&frame.order.unwrap().chosen, recipient);
        if !attached.is_empty()
            && let Some(AttachmentCallback::Reflexive(ability)) = frame.callback
        {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            self.queue_reflexive_objects(
                &item,
                ability,
                std::iter::once(recipient).chain(attached),
            );
        }
        false
    }
    pub(super) fn visit_return_attachment_actions(
        &self,
        player: Player,
        mut emit: impl FnMut(Action) -> bool,
    ) -> bool {
        let frame = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .return_attachment
            .as_ref()
            .unwrap();
        if let Some(order) = &frame.order {
            return order.remaining[player.index()].iter().any(|&equipment| {
                emit(Action::ChooseEquipmentAttachment {
                    equipment: Some(equipment),
                })
            });
        }
        if !frame.required && emit(Action::ChooseEquipmentAttachment { equipment: None }) {
            return true;
        }
        self.visit_attachment_candidates(frame, |equipment| {
            emit(Action::ChooseEquipmentAttachment {
                equipment: Some(equipment),
            })
        })
    }
    pub(super) fn choose_equipment_attachment(
        &mut self,
        player: Player,
        equipment: Option<ObjectRef>,
    ) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let frame = resolution.return_attachment.as_mut().unwrap();
        if let Some(order) = &mut frame.order {
            let equipment = equipment.expect("mandatory timestamp ordering");
            let remaining = &mut order.remaining[player.index()];
            remaining.retain(|r| *r != equipment);
            order.chosen.push(equipment);
            if remaining.len() <= 1 {
                self.run_program();
            }
            return;
        }
        if let (Some(selected), Some(equipment)) = (&mut frame.selected, equipment) {
            selected.push(equipment);
            return;
        }
        let frame = resolution.return_attachment.take().unwrap();
        if let Some(recipient) = frame.recipient {
            if let Some(mut selected) = frame.selected {
                // Reattaching to the same creature changes neither attachment nor timestamp.
                selected.retain(|&equipment| {
                    self.equipment_can_attach(equipment, recipient)
                        && self.state.objects[equipment.card as usize].attached_to
                            != Some(recipient)
                });
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                if self.order_equipment_attachments(&item, recipient, selected) {
                    return;
                }
            } else if let Some(equipment) = equipment
                && self.attach_equipment(equipment, recipient)
                && let Some(AttachmentCallback::UnattachAtNextEndStep(ability)) = frame.callback
            {
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                self.schedule_delayed_step(
                    &item,
                    TriggerStep::End,
                    CastRelation::Any,
                    ability,
                    Some(equipment),
                    None,
                );
            }
        }
        self.run_program();
    }
    pub(super) fn equipment_attachment_view(&self) -> Option<EquipmentAttachmentView> {
        let Pending::EquipmentAttachment { player } = self.state.pending else {
            return None;
        };
        let frame = self.state.resolving.as_ref()?.return_attachment.as_ref()?;
        let recipient = frame.recipient?;
        let (candidates, prior_public_order) = if let Some(order) = &frame.order {
            (
                order.remaining[player.index()].clone(),
                order.chosen.clone(),
            )
        } else {
            let mut candidates = Vec::new();
            self.visit_attachment_candidates(frame, |equipment| {
                candidates.push(equipment);
                false
            });
            (candidates, Vec::new())
        };
        Some(EquipmentAttachmentView {
            player,
            recipient: self.object_view(recipient.card),
            ordering: frame.order.as_ref().map(|_| true),
            candidates,
            required: frame.required,
            selected: frame.selected.clone(),
            prior_public_order,
        })
    }
}
