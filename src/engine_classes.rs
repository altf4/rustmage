//! Class levels are exact-permanent designations, never counters or copiable values.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ClassLevelView {
    pub source: ObjectRef,
    pub level: u8,
}
impl Game {
    pub(super) fn class_level(&self, source: ObjectRef) -> u8 {
        self.state
            .class_levels
            .as_ref()
            .and_then(|rows| rows.iter().find(|row| row.source == source))
            .map_or(1, |row| row.level)
    }
    pub(super) fn class_activation_available(&self, card: CardId, ability: usize) -> bool {
        self.state.objects[card as usize]
            .card
            .class_level_activations()
            .iter()
            .find(|&&(index, _)| usize::from(index) == ability)
            .is_none_or(|&(_, level)| self.class_level(self.object_ref(card)) == level - 1)
    }
    pub(super) fn set_class_level(&mut self, item: &SpellView, level: u8) {
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
        {
            return;
        }
        if self.class_level(item.object) == level {
            return;
        }
        let rows = self.state.class_levels.get_or_insert_with(Default::default);
        if let Some(row) = rows.iter_mut().find(|row| row.source == item.object) {
            row.level = level;
        } else {
            rows.try_reserve(1)
                .capacity("Class level designation allocation exhausted");
            rows.push(ClassLevelView {
                source: item.object,
                level,
            });
        }
        let source = self.object_view(item.object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if matches!(ability.condition.event(),TriggerCondition::ClassLevel {level: required} if required == level)
            {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn visible_class_levels(&self) -> Vec<ClassLevelView> {
        self.state
            .class_levels
            .as_ref()
            .map_or_else(Vec::new, |rows| {
                rows.iter()
                    .copied()
                    .filter(|row| {
                        self.on_battlefield(row.source.card)
                            && self.object_ref(row.source.card) == row.source
                    })
                    .collect()
            })
    }
    pub(super) fn prune_class_levels(&mut self) {
        let objects = &self.state.objects;
        if let Some(rows) = &mut self.state.class_levels {
            rows.retain(|row| {
                let o = &objects[row.source.card as usize];
                o.zone == Zone::Battlefield && o.generation == row.source.generation
            });
        }
    }
}

impl Game {
    pub(super) fn freeze_departure_sources(&self) -> Option<SharedVec<ObjectView>> {
        let sources: Vec<_> = self
            .state
            .death_sources
            .iter()
            .filter_map(|&id| {
                let o = &self.state.objects[id as usize];
                (o.zone == Zone::Battlefield
                    && o.card
                        .triggers()
                        .iter()
                        .any(|a| a.condition.departure_filter().is_some()))
                .then(|| self.object_view(id))
            })
            .collect();
        (!sources.is_empty()).then(|| sources.into())
    }
}
