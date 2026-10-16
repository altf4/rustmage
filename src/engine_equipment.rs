//! Exact-incarnation attachment links and continuous Equipment bonuses.
use super::*;
impl Game {
    pub(super) fn attached_subject(&self, item: &SpellView) -> Option<CardId> {
        let target = if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object
        {
            self.state.objects[item.card as usize].attached_to
        } else {
            item.source_attachment
        }?;
        (self.on_battlefield(target.card)
            && self.object_ref(target.card) == target
            && self.state.objects[target.card as usize].card.is_creature())
        .then_some(target.card)
    }

    pub(super) fn attachment_bonus(&self, id: CardId) -> (Stats, Keywords) {
        let mut stats = Stats::default();
        let mut keywords = Keywords::NONE;
        if !self.on_battlefield(id) || !self.state.objects[id as usize].card.is_creature() {
            return (stats, keywords);
        }
        let target = self.object_ref(id);
        for &source in &self.state.attachment_sources {
            let object = &self.state.objects[source as usize];
            if object.attached_to == Some(target) && !object.card.is_creature() {
                let spec = object.card.attachment_bonus().unwrap();
                stats += spec.stats;
                keywords = keywords.union(spec.keywords);
            }
        }
        (stats, keywords)
    }

    pub(super) fn keywords(&self, id: CardId) -> Keywords {
        let target = &self.state.objects[id as usize];
        let mut keywords = target.keywords().union(self.attachment_bonus(id).1);
        if target.zone != Zone::Battlefield {
            return keywords;
        }
        let reference = self.object_ref(id);
        let attacking = self.state.attackers.iter().any(|a| a.card == id);
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            for grant in object.card.keyword_grants() {
                if grant.matches(
                    target.card,
                    target.owner == object.owner,
                    reference == self.object_ref(source),
                    attacking,
                    object.owner == self.state.active,
                ) {
                    keywords = keywords.union(grant.keywords);
                }
            }
        }
        keywords
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
