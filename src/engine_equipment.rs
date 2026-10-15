//! Exact-incarnation attachment links and continuous Equipment bonuses.
use super::*;
impl Game {
    pub(super) fn equipment_bonus(&self, id: CardId) -> (Stats, Keywords) {
        let mut stats = Stats::default();
        let mut keywords = Keywords::NONE;
        if !self.on_battlefield(id) || !self.state.objects[id as usize].card.is_creature() {
            return (stats, keywords);
        }
        let target = self.object_ref(id);
        for &source in &self.state.equipment_sources {
            let object = &self.state.objects[source as usize];
            if object.attached_to == Some(target) && !object.card.is_creature() {
                let spec = object.card.equipment().unwrap();
                stats += spec.stats;
                keywords = keywords.union(spec.keywords);
            }
        }
        (stats, keywords)
    }

    pub(super) fn keywords(&self, id: CardId) -> Keywords {
        self.state.objects[id as usize]
            .keywords()
            .union(self.equipment_bonus(id).1)
    }

    pub(super) fn attach_equipment(&mut self, source: ObjectRef, target: ObjectRef) -> bool {
        if source == target
            || !self.on_battlefield(source.card)
            || self.object_ref(source.card) != source
            || self.state.objects[source.card as usize]
                .card
                .equipment()
                .is_none()
            || self.state.objects[source.card as usize].card.is_creature()
            || !self.on_battlefield(target.card)
            || self.object_ref(target.card) != target
            || !self.state.objects[target.card as usize].card.is_creature()
        {
            return false;
        }
        self.state.objects[source.card as usize].attached_to = Some(target);
        true
    }
}
