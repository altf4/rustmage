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
                    && object.card.aura().is_some()
                    && object.attached_to == Some(self.object_ref(id))
            })
    }

    pub(super) fn attached_permanent(&self, item: &SpellView) -> Option<CardId> {
        let target = if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object
        {
            self.state.objects[item.card as usize].attached_to
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

    pub(super) fn attach_equipment(&mut self, source: ObjectRef, target: ObjectRef) -> bool {
        if source == target
            || !self.on_battlefield(source.card)
            || self.object_ref(source.card) != source
            || self.state.objects[source.card as usize]
                .card
                .equipment()
                .is_none()
            || self.is_creature(source.card)
            || !self.on_battlefield(target.card)
            || self.object_ref(target.card) != target
            || !self.is_creature(target.card)
            || self.protected_from_colors(
                target.card,
                ability_world::CharacteristicWorld::colors(self, source.card),
            )
        {
            return false;
        }
        if self.state.objects[source.card as usize].attached_to != Some(target) {
            self.retimestamp_live_static_source(source);
        }
        self.state.objects[source.card as usize].attached_to = Some(target);
        true
    }
}
