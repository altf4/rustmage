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
        if triggers
            .iter()
            .any(|a| a.condition.event() == TriggerCondition::EvidenceCollected)
        {
            self.register_evidence_source(id);
        }
        if triggers
            .iter()
            .any(|a| a.condition.event() == TriggerCondition::CreatureExploresYou)
        {
            self.register_explore_source(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::AttackDeclared { .. }
            )
        }) {
            self.state.attack_sources.push(id);
        }
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
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Sacrificed { .. }
            )
        }) {
            self.state.death_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| {
                matches!(
                    ability.condition.event(),
                    TriggerCondition::AtStep { .. } | TriggerCondition::Chapter { .. }
                )
            })
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
            .replacement_effects()
            .is_empty()
            || !self.state.objects[id as usize]
                .card
                .enchanted_modifiers()
                .is_empty()
            || !self.state.objects[id as usize]
                .card
                .static_modifiers()
                .is_empty()
            || !self.state.objects[id as usize]
                .card
                .keyword_grants()
                .is_empty()
            || !self.state.objects[id as usize]
                .card
                .counter_keyword_grants()
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
            || self.state.objects[id as usize]
                .card
                .entry_trigger_suppression()
                != 0
            || self.state.objects[id as usize].card.opponent_entry_tapped() != 0
            || !self.state.objects[id as usize]
                .card
                .cast_timing_grants()
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
            && self.is_creature(id)
            && filter.matches_characteristics(
                object.card,
                self.current_kinds(id),
                self.granted_subtypes(id),
                object.controller == controller,
                self.object_ref(id) == source,
                self.state.attackers.iter().any(|a| a.card == id),
            )
            && (filter.keyword == Keywords::NONE || self.keywords(id).contains(filter.keyword))
    }

    pub(super) fn stats(&self, id: CardId) -> Stats {
        self.stats_without_hand(id, None)
    }
    pub(super) fn stats_without_hand(&self, id: CardId, departing: Option<CardId>) -> Stats {
        let object = &self.state.objects[id as usize];
        let mut stats = if let Some((power, toughness)) = object.card.vehicle_stats() {
            if !self.is_creature(id) {
                return Stats::default();
            }
            Stats {
                power: power
                    .checked_add(object.growth.power)
                    .and_then(|n| n.checked_add(object.counter_delta()))
                    .capacity("Vehicle power exhausted"),
                toughness: toughness
                    .checked_add(object.growth.toughness)
                    .and_then(|n| n.checked_add(object.counter_delta()))
                    .capacity("Vehicle toughness exhausted"),
            }
        } else {
            Stats {
                power: object.local_power(),
                toughness: object.local_toughness(),
            }
        };
        // Characteristic-defining abilities apply in every zone (layer 7a).
        if let Some(base) = object.card.dynamic_stats() {
            let reference = self.object_ref(id);
            let power = base.power.map(|expr| {
                self.stat_expression_without_hand(expr, reference, object.controller, departing)
            });
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
                    self.stat_expression_without_hand(expr, reference, object.controller, departing)
                };
                stats.toughness = value
                    .checked_add(object.growth.toughness)
                    .and_then(|t| t.checked_add(object.counter_delta()))
                    .capacity("toughness exhausted");
            }
        }
        if object.zone != Zone::Battlefield || !self.is_creature(id) {
            return stats;
        }
        // Base-setting effects apply after CDAs and before all layer-7c additions.
        if self.has_cursed_role(self.object_ref(id)) {
            stats = Stats {
                power: 1i128
                    .checked_add(object.growth.power)
                    .and_then(|v| v.checked_add(object.counter_delta()))
                    .capacity("Cursed power exhausted"),
                toughness: 1i128
                    .checked_add(object.growth.toughness)
                    .and_then(|v| v.checked_add(object.counter_delta()))
                    .capacity("Cursed toughness exhausted"),
            };
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
                        !self.is_creature(source) && object.attached_to == Some(self.object_ref(id))
                    }
                    Subject::Targets | Subject::Entering => {
                        unreachable!("event or targeted scaling static modifier")
                    }
                };
                if applies {
                    let value = self.stat_value_without_hand(
                        modifier.value,
                        self.object_ref(source),
                        object.controller,
                        departing,
                    );
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
                if (!modifier.own_turn || self.state.active == object.controller)
                    && (!modifier.requires_life_gain
                        || self.state.players[object.controller.index()].life_gained_this_turn > 0)
                    && self.creature_matches(
                        modifier.filter,
                        object.controller,
                        self.object_ref(source),
                        id,
                    )
                {
                    stats += modifier.stats;
                }
            }
            if !object.card.enchanted_modifiers().is_empty() && self.is_enchanted(id) {
                for modifier in object.card.enchanted_modifiers() {
                    if self.creature_matches(
                        modifier.filter,
                        object.controller,
                        self.object_ref(source),
                        id,
                    ) {
                        stats += modifier.stats;
                    }
                }
            }
        }
        stats
    }
}
