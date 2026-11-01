//! Exact successful return results and public, nontargeting Equipment choices.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReturnAttachment {
    pub origin: ObjectRef,
    pub player: Player,
    pub recipient: Option<ObjectRef>,
    pub entering: bool,
    order: Option<Box<EquipmentOrder>>,
    reflexive: Option<u16>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct EquipmentOrder {
    remaining: Vec<ObjectRef>,
    chosen: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ReturnAttachment {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.order.as_ref().map_or(0, |order| {
                std::mem::size_of::<EquipmentOrder>()
                    + (order.remaining.len() + order.chosen.len())
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
    /// Mandatory relative timestamp choice for an already selected batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordering: Option<bool>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn may_attach_controlled_equipment(
        &mut self,
        item: &SpellView,
        group: u16,
        subtype: Option<&'static str>,
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
                    && subtype.is_none_or(|t| self.has_subtype(r.card, t))
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
                reflexive: None,
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
            if self.attach_equipment_batch(&equipment, recipient) != 0
                && let Some(ability) = reflexive
            {
                self.queue_reflexive(item, ability, Some(recipient));
            }
            return false;
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
                reflexive,
                order: Some(Box::new(EquipmentOrder {
                    remaining: equipment,
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
                reflexive: None,
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
    fn return_attachment_candidate(&self, equipment: ObjectRef) -> bool {
        let Some(frame) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.return_attachment.as_ref())
        else {
            return false;
        };
        frame
            .order
            .as_ref()
            .is_none_or(|order| order.remaining.contains(&equipment))
            && frame.recipient.is_some_and(|recipient| {
                self.state
                    .objects
                    .get(equipment.card as usize)
                    .is_some_and(|o| o.controller == frame.player)
                    && self.equipment_can_attach(equipment, recipient)
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
            self.state.pending = Pending::EquipmentAttachment {
                player: frame.player,
            };
            return true;
        }
        if frame.recipient.is_some_and(|recipient| {
            self.on_battlefield(recipient.card)
                && self.object_ref(recipient.card) == recipient
                && self.is_creature(recipient.card)
        }) && self
            .state
            .objects
            .iter()
            .enumerate()
            .any(|(id, _)| self.return_attachment_candidate(self.object_ref(id as CardId)))
        {
            self.state.pending = Pending::EquipmentAttachment {
                player: frame.player,
            };
            return true;
        }
        self.state.resolving.as_mut().unwrap().return_attachment = None;
        false
    }
    pub(super) fn visit_return_attachment_actions(
        &self,
        mut emit: impl FnMut(Action) -> bool,
    ) -> bool {
        let ordering = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.return_attachment.as_ref())
            .is_some_and(|frame| frame.order.is_some());
        if !ordering && emit(Action::ChooseEquipmentAttachment { equipment: None }) {
            return true;
        }
        for (id, _) in self.state.objects.iter().enumerate() {
            let equipment = self.object_ref(id as CardId);
            if self.return_attachment_candidate(equipment)
                && emit(Action::ChooseEquipmentAttachment {
                    equipment: Some(equipment),
                })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn choose_equipment_attachment(&mut self, equipment: Option<ObjectRef>) {
        let ordering = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .return_attachment
            .as_ref()
            .unwrap()
            .order
            .is_some();
        if ordering {
            let order = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .return_attachment
                .as_mut()
                .unwrap()
                .order
                .as_mut()
                .unwrap();
            let equipment = equipment.expect("mandatory timestamp ordering");
            order.remaining.retain(|r| *r != equipment);
            order.chosen.push(equipment);
            if order.remaining.len() > 1 {
                return;
            }
            order.chosen.append(&mut order.remaining);
            let frame = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .return_attachment
                .take()
                .unwrap();
            let recipient = frame.recipient.unwrap();
            let equipment: Vec<_> = frame
                .order
                .unwrap()
                .chosen
                .into_iter()
                .filter(|r| self.state.objects[r.card as usize].controller == frame.player)
                .collect();
            if self.attach_equipment_batch(&equipment, recipient) != 0
                && let Some(ability) = frame.reflexive
            {
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                self.queue_reflexive(&item, ability, Some(recipient));
            }
            self.run_program();
            return;
        }
        let frame = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .return_attachment
            .take()
            .unwrap();
        if let (Some(equipment), Some(recipient)) = (equipment, frame.recipient) {
            self.attach_equipment(equipment, recipient);
        }
        self.run_program();
    }
    pub(super) fn equipment_attachment_view(&self) -> Option<EquipmentAttachmentView> {
        if !matches!(self.state.pending, Pending::EquipmentAttachment { .. }) {
            return None;
        }
        let frame = self.state.resolving.as_ref()?.return_attachment.as_ref()?;
        let recipient = frame.recipient?;
        Some(EquipmentAttachmentView {
            player: frame.player,
            recipient: self.object_view(recipient.card),
            ordering: frame.order.as_ref().map(|_| true),
            candidates: self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, _)| {
                    let r = self.object_ref(id as CardId);
                    self.return_attachment_candidate(r).then_some(r)
                })
                .collect(),
        })
    }
}
