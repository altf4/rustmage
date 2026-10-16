//! Derived additive continuous effects. Unsupported setting/switching/dependency
//! layers remain gated by complete-text admission, never approximated by deltas.
use super::*;

impl Game {
    pub(super) fn register_battlefield_sources(&mut self, id: CardId) {
        if self.state.objects[id as usize]
            .card
            .attachment_bonus()
            .is_some()
        {
            self.state.attachment_sources.push(id);
        }
        let triggers = self.state.objects[id as usize].card.triggers();
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::CardDrawn { .. }
            )
        }) {
            self.state.draw_sources.push(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::Discarded { .. }
            )
        }) {
            self.state.discard_sources.push(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::PlayerDamaged { .. }
            )
        }) {
            self.state.damage_sources.push(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::LifeGained { .. }
            )
        }) {
            self.state.life_sources.push(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::SelfDies
                    | TriggerCondition::SelfEntersOrDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::Sacrificed { .. }
            )
        }) {
            self.state.death_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| matches!(ability.condition.event(), TriggerCondition::AtStep { .. }))
        {
            self.state.step_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| {
                matches!(
                    ability.condition.event(),
                    TriggerCondition::SpellCast { .. }
                )
            })
        {
            self.state.cast_sources.push(id);
        }
        if !self.state.objects[id as usize]
            .card
            .static_modifiers()
            .is_empty()
            || !self.state.objects[id as usize]
                .card
                .keyword_grants()
                .is_empty()
            || !self.state.objects[id as usize]
                .card
                .scaling_modifiers()
                .is_empty()
            || self.state.objects[id as usize]
                .card
                .cost_reductions()
                .iter()
                .any(|rule| !rule.source_only)
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
            && filter.matches_intrinsic(
                object.card,
                object.owner == controller,
                self.object_ref(id) == source,
                self.state.attackers.iter().any(|a| a.card == id),
            )
            && (filter.keyword == Keywords::NONE || self.keywords(id).contains(filter.keyword))
    }

    pub(super) fn stats(&self, id: CardId) -> Stats {
        let object = &self.state.objects[id as usize];
        let mut stats = Stats {
            power: object.local_power(),
            toughness: object.local_toughness(),
        };
        // Characteristic-defining abilities apply in every zone (layer 7a).
        if let Some(base) = object.card.dynamic_stats() {
            let reference = self.object_ref(id);
            let power = base
                .power
                .map(|expr| self.stat_expression(expr, reference, object.owner));
            if let Some(power) = power {
                stats.power = power
                    .checked_add(object.growth.power)
                    .and_then(|p| p.checked_add(object.counter_delta()))
                    .capacity("power exhausted");
            }
            if let Some(expr) = base.toughness {
                // Most CDAs share a count for both axes; query that count once.
                let value = if let Some(p) = base.power
                    && p.value == expr.value
                    && p.multiplier == expr.multiplier
                {
                    power
                        .unwrap()
                        .checked_add(expr.offset)
                        .and_then(|v| v.checked_sub(p.offset))
                        .capacity("toughness exhausted")
                } else {
                    self.stat_expression(expr, reference, object.owner)
                };
                stats.toughness = value
                    .checked_add(object.growth.toughness)
                    .and_then(|t| t.checked_add(object.counter_delta()))
                    .capacity("toughness exhausted");
            }
        }
        if object.zone != Zone::Battlefield || !object.card.is_creature() {
            return stats;
        }
        stats += self.attachment_bonus(id).0;
        // Only the small live-source index is traversed. Central zone transitions
        // maintain it, and snapshots own it; no derived stat cache needs invalidation.
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            for modifier in object.card.scaling_modifiers() {
                let applies = match modifier.subject {
                    Subject::Source => source == id,
                    Subject::Attached => {
                        !object.card.is_creature()
                            && object.attached_to == Some(self.object_ref(id))
                    }
                    Subject::Targets => unreachable!("targeted scaling static modifier"),
                };
                if applies {
                    let value =
                        self.stat_value(modifier.value, self.object_ref(source), object.owner);
                    stats += Stats {
                        power: modifier
                            .stats
                            .power
                            .checked_mul(value)
                            .capacity("power exhausted"),
                        toughness: modifier
                            .stats
                            .toughness
                            .checked_mul(value)
                            .capacity("toughness exhausted"),
                    };
                }
            }
            for modifier in object.card.static_modifiers() {
                if (!modifier.requires_life_gain
                    || self.state.players[object.owner.index()].life_gained_this_turn > 0)
                    && self.creature_matches(
                        modifier.filter,
                        object.owner,
                        self.object_ref(source),
                        id,
                    )
                {
                    stats += modifier.stats;
                }
            }
        }
        stats
    }
}
