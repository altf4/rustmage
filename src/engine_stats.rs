//! Derived layer-7 statistics: characteristic definitions, timestamped base
//! setters, then additive modifiers. Unsupported switching/dependency effects
//! remain gated by complete-text admission, never approximated by deltas.
use super::*;

impl Game {
    pub(super) fn register_battlefield_sources(&mut self, id: CardId) {
        self.register_static_base_stats(id);
        self.register_live_static_source(id);
        self.register_ability_source(id);
        if self.state.objects[id as usize]
            .card
            .attachment_bonus()
            .is_some()
        {
            self.state.attachment_sources.push(id);
        }
        let triggers = self.state.objects[id as usize].card.triggers();
        if triggers.iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::OtherCreatureCountersAdded { .. }
            )
        }) {
            self.state.counter_sources.push(id);
        }
        if triggers
            .iter()
            .any(|a| a.condition.event() == TriggerCondition::EvidenceCollected)
        {
            self.register_evidence_source(id);
        }
        if triggers.iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::CreatureExploresYou
                    | TriggerCondition::CreatureExploresCardYou { .. }
            )
        }) {
            self.register_explore_source(id);
        }
        if triggers.iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::CreatureAttacksYou { .. }
                    | TriggerCondition::AttackDeclared { .. }
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
                TriggerCondition::LifeGained { .. } | TriggerCondition::LifeLost { .. }
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
        if self.state.objects[id as usize]
            .card
            .static_ability_groups()
            .iter()
            .any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::SelfUntap(_)
                            | crate::cards::StaticAbilityFragment::LifeGainAdditional(_)
                            | crate::cards::StaticAbilityFragment::CantGainLife
                            | crate::cards::StaticAbilityFragment::DeathBaronLord
                            | crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(..)
                    )
                })
            })
            || !self.state.objects[id as usize]
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
            || self.state.objects[id as usize]
                .card
                .controlled_entry_untapped()
                != 0
            || self.state.objects[id as usize]
                .card
                .static_cast_restriction()
                != 0
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
        self.characteristic_query()
            .creature_matches(filter, controller, source, id)
    }

    pub(super) fn stats(&self, id: CardId) -> Stats {
        self.characteristic_query().stats(id)
    }
    // CR208.4b: characteristic definitions and setters, before modifiers/counters.
    #[cfg(test)]
    pub(super) fn base_stats_without_hand(&self, id: CardId, departing: Option<CardId>) -> Stats {
        self.characteristic_query()
            .base_stats_without_hand(id, departing)
    }

    pub(super) fn stats_without_hand(&self, id: CardId, departing: Option<CardId>) -> Stats {
        self.characteristic_query()
            .stats_without_hand(id, departing)
    }
    pub(super) fn layered_stats_without_hand(
        &self,
        id: CardId,
        departing: Option<CardId>,
    ) -> (Stats, Stats) {
        self.characteristic_query()
            .layered_stats_without_hand(id, departing)
    }
}
