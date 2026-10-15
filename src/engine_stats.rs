//! Derived additive continuous effects. Unsupported setting/switching/dependency
//! layers remain gated by complete-text admission, never approximated by deltas.
use super::*;

impl Game {
    pub(super) fn register_battlefield_sources(&mut self, id: CardId) {
        let triggers = self.state.objects[id as usize].card.triggers();
        if triggers
            .iter()
            .any(|ability| matches!(ability.condition, TriggerCondition::LifeGained { .. }))
        {
            self.state.life_sources.push(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition,
                TriggerCondition::SelfDies | TriggerCondition::CreatureDies { .. }
            )
        }) {
            self.state.death_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| matches!(ability.condition, TriggerCondition::AtStep { .. }))
        {
            self.state.step_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| matches!(ability.condition, TriggerCondition::SpellCast { .. }))
        {
            self.state.cast_sources.push(id);
        }
        if !self.state.objects[id as usize]
            .card
            .static_modifiers()
            .is_empty()
        {
            self.state.static_sources.push(id);
        }
    }

    pub(super) fn creature_matches(
        &self,
        filter: CreatureFilter,
        controller: Player,
        source: ObjectRef,
        id: CardId,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        object.zone == Zone::Battlefield
            && object.card.is_creature()
            && match filter.relation {
                0 => object.owner == controller,
                1 => object.owner != controller,
                _ => true,
            }
            && (!filter.other || self.object_ref(id) != source)
            && filter.subtype.is_none_or(|subtype| {
                object
                    .card
                    .info()
                    .type_line
                    .split_once(" — ")
                    .is_some_and(|(_, types)| types.split_whitespace().any(|word| word == subtype))
            })
            && object.keywords().contains(filter.keyword)
            && (!filter.attacking || self.state.attackers.iter().any(|a| a.card == id))
            && filter
                .tokens
                .is_none_or(|tokens| object.card.is_token() == tokens)
            && (!filter.artifact || object.card.is_artifact())
    }

    pub(super) fn stats(&self, id: CardId) -> Stats {
        let object = &self.state.objects[id as usize];
        let mut stats = Stats {
            power: object.local_power(),
            toughness: object.local_toughness(),
        };
        if object.zone != Zone::Battlefield || !object.card.is_creature() {
            return stats;
        }
        // Only the small live-source index is traversed. Central zone transitions
        // maintain it, and snapshots own it; no derived stat cache needs invalidation.
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            for modifier in object.card.static_modifiers() {
                if self.creature_matches(modifier.filter, object.owner, self.object_ref(source), id)
                {
                    stats += modifier.stats;
                }
            }
        }
        stats
    }
}
