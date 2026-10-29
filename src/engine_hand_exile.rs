//! Exact hand/discard incarnations and independent ordinary paid exile permissions.
use super::*;

impl Game {
    pub(super) fn exile_hand_draw(&mut self, item: &SpellView) {
        let player = item.controller;
        let cards: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.zone == Zone::Hand && object.owner == player && object.is_card()
            })
            .map(|(index, _)| index as CardId)
            .collect();
        let amount = cards.len() as u128;
        let events = self.move_objects_raw(&cards, Zone::Exile);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Exile)
            .map(|event| event.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.draw_many(player, amount);
    }

    pub(super) fn grant_exiled_result_play(&mut self, player: Player, next_turn: bool) {
        let objects = std::mem::take(&mut self.state.resolving.as_mut().unwrap().milled);
        for object in objects {
            self.grant_exile_play_permission(player, object, next_turn);
        }
    }

    pub(super) fn discarded_play_object(&self, item: &SpellView) -> Option<ObjectRef> {
        let reference = item.reflexive_subject.as_ref()?.object;
        let object = self.state.objects.get(reference.card as usize)?;
        (self.object_ref(reference.card) == reference
            && object.zone == Zone::Graveyard
            && object.owner == item.controller
            && object.is_card())
        .then_some(reference)
    }

    pub(super) fn complete_optional_discard_exile_play(&mut self) {
        let resolution = self.state.resolving.as_ref().unwrap();
        let Some(Instruction::MayExileDiscardedPlay { next_turn }) =
            resolution.program.get(resolution.pc.saturating_sub(1))
        else {
            return;
        };
        let (item, next_turn) = (resolution.item.clone(), *next_turn);
        let Some(object) = self.discarded_play_object(&item) else {
            return;
        };
        let events = self.move_objects_raw(&[object.card], Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(item.controller, event.after, next_turn);
            }
        }
        self.finish_zone_events(&events, &[]);
    }
}
