//! Saga chapters are ordinary exact-source triggered packets, not mutable scripts.
use super::*;
impl Game {
    /// CR714.3c/714.4 require a live Saga with at least one chapter ability.
    /// The admitted chapter set is its immutable definition, subject to live removal.
    fn live_saga_final_chapter(&self, id: CardId) -> Option<u16> {
        let object = &self.state.objects[id as usize];
        (object.zone == Zone::Battlefield
            && self.has_subtype(id, "Saga")
            && self.printed_abilities_available(id))
        .then(|| object.card.saga_final_chapter())
        .flatten()
    }
    pub(super) fn lore_counters_added(&mut self, object: ObjectRef, before: u64) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let after = self.state.objects[object.card as usize].counters.lore;
        if after <= before {
            return;
        }
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if let TriggerCondition::Chapter { number } = ability.condition
                && before < u64::from(number)
                && u64::from(number) <= after
            {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn precombat_lore_counters(&mut self) {
        let placements = self
            .state
            .step_sources
            .iter()
            .filter_map(|&id| {
                let o = &self.state.objects[id as usize];
                if o.zone != Zone::Battlefield
                    || o.controller != self.state.active
                    || self.live_saga_final_chapter(id).is_none()
                {
                    return None;
                }
                Some(replacements::CounterPlacement {
                    placed_by: o.controller,
                    object: self.object_ref(id),
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: self.granted_subtypes(id),
                    kind: self.current_kinds(id),
                    amount: Counters {
                        lore: 1,
                        ..Counters::default()
                    },
                    effect: false,
                    entry_index: None,
                    private: false,
                })
            })
            .collect();
        self.stage_counter_placements(placements, replacements::ResumeMode::Pending);
    }
    fn is_chapter_packet(item: &SpellView, source: ObjectRef) -> bool {
        item.object == source
            && match item.kind {
                StackKind::TriggeredAbility { ability } => item
                    .trigger_definition()
                    .triggers()
                    .get(ability as usize)
                    .is_some_and(|a| matches!(a.condition, TriggerCondition::Chapter { .. })),
                _ => false,
            }
    }
    pub(super) fn saga_sacrifice_required(&self, id: CardId) -> bool {
        let o = &self.state.objects[id as usize];
        let Some(final_chapter) = self.live_saga_final_chapter(id) else {
            return false;
        };
        if o.counters.lore < u64::from(final_chapter) {
            return false;
        }
        let source = self.object_ref(id);
        !self
            .state
            .stack
            .iter()
            .chain(self.state.pending_triggers.iter())
            .chain(self.state.ordering_triggers.iter())
            .any(|item| Self::is_chapter_packet(item, source))
            && !self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| Self::is_chapter_packet(&a.item, source))
            && !self
                .state
                .resolving
                .as_ref()
                .is_some_and(|r| Self::is_chapter_packet(&r.item, source))
    }
}
