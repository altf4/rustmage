//! Exact successful return results and public, nontargeting Equipment choices.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReturnAttachment {
    pub origin: ObjectRef,
    pub player: Player,
    pub recipient: Option<ObjectRef>,
    pub entering: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct EquipmentAttachmentView {
    pub player: Player,
    pub recipient: ObjectView,
    pub candidates: Vec<ObjectRef>,
}
impl Game {
    pub(super) fn return_source_with_equipment(&mut self, item: &SpellView, counter: CounterKind) {
        if self.object_ref(item.card) != item.object
            || self.state.objects[item.card as usize].zone != Zone::Graveyard
        {
            return;
        }
        self.state.resolving.as_mut().unwrap().return_attachment =
            Some(Box::new(ReturnAttachment {
                origin: item.object,
                player: item.controller,
                recipient: None,
                entering: true,
            }));
        let mut movement = self.entry_move(item.card, Zone::Battlefield, false);
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
        frame.recipient.is_some_and(|recipient| {
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
        if emit(Action::ChooseEquipmentAttachment { equipment: None }) {
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
