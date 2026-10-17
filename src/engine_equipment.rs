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

    pub(super) fn attached_subject(&self, item: &SpellView) -> Option<CardId> {
        let target = if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object
        {
            self.state.objects[item.card as usize].attached_to
        } else {
            item.source_attachment
        }?;
        (self.on_battlefield(target.card)
            && self.object_ref(target.card) == target
            && self.is_creature(target.card))
        .then_some(target.card)
    }

    pub(super) fn attachment_bonus(&self, id: CardId) -> (Stats, Keywords) {
        let mut stats = Stats::default();
        let mut keywords = Keywords::NONE;
        if !self.on_battlefield(id) || !self.is_creature(id) {
            return (stats, keywords);
        }
        let target = self.object_ref(id);
        for &source in &self.state.attachment_sources {
            let object = &self.state.objects[source as usize];
            if object.attached_to == Some(target) && !self.is_creature(source) {
                let spec = object.card.attachment_bonus().unwrap();
                stats += spec.stats;
                keywords = keywords.union(spec.keywords);
                if object.controller == self.state.active {
                    keywords = keywords.union(object.card.attached_turn_keywords());
                }
            }
        }
        (stats, keywords)
    }

    pub(super) fn keywords(&self, id: CardId) -> Keywords {
        if self.state.objects[id as usize].zone == Zone::Stack {
            let definition = self.effective_spell_card(id);
            if definition.inset_parent().is_some() {
                return definition.info().keywords;
            }
        }
        let target = &self.state.objects[id as usize];
        let mut keywords = target.keywords().union(self.attachment_bonus(id).1);
        if target.zone != Zone::Battlefield {
            return keywords;
        }
        if self.is_suspected(id) {
            keywords = keywords.union(Keywords::MENACE).union(Keywords::CANT_BLOCK);
        }
        let reference = self.object_ref(id);
        let attacking = self.state.attackers.iter().any(|a| a.card == id);
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            if self.is_creature(id) && target.controller == object.controller {
                for &(kind, grant) in object.card.counter_keyword_grants() {
                    if target.counters.get(kind) > 0 {
                        keywords = keywords.union(grant);
                    }
                }
            }
            for grant in object.card.keyword_grants() {
                if grant.condition.is_none_or(|value| {
                    self.stat_value(value, self.object_ref(source), object.controller) != 0
                }) && (!grant.requires_life_gain
                    || self.state.players[object.controller.index()].life_gained_this_turn > 0)
                    && grant.matches_characteristics(
                        target.card,
                        self.current_kinds(id),
                        self.granted_subtypes(id),
                        target.controller == object.controller,
                        reference == self.object_ref(source),
                        attacking,
                        object.controller == self.state.active,
                    )
                {
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
            || self.is_creature(source.card)
            || !self.on_battlefield(target.card)
            || self.object_ref(target.card) != target
            || !self.is_creature(target.card)
        {
            return false;
        }
        self.state.objects[source.card as usize].attached_to = Some(target);
        true
    }
}
