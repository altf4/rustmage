//! Derived layer-7 statistics: characteristic definitions, timestamped base
//! setters, then additive modifiers. Unsupported switching/dependency effects
//! remain gated by complete-text admission, never approximated by deltas.
use super::*;

impl Game {
    // Reuse sparse event indexes for explicit graveyard observers.
    pub(super) fn register_graveyard_sources(&mut self, id: CardId) {
        let triggers = self.state.objects[id as usize].card.triggers();
        if triggers.iter().any(|a| {
            a.condition.origin() == Some(TargetZone::Graveyard)
                && matches!(a.condition.event(), TriggerCondition::AtStep { .. })
        }) {
            self.state.step_sources.push(id);
        }
        if triggers.iter().any(|a| {
            a.condition.origin() == Some(TargetZone::Graveyard)
                && matches!(a.condition.event(), TriggerCondition::PlayerDamaged { .. })
        }) {
            self.state.damage_sources.push(id);
        }
    }
    pub(super) fn register_battlefield_sources(&mut self, id: CardId) {
        let object = self.object_ref(id);
        let entries = self.state.entered_this_turn.get_or_insert_default();
        if !entries.contains(&object) {
            entries.push(object);
        }
        self.register_battlefield_providers(id);
    }
    pub(super) fn register_battlefield_providers(&mut self, id: CardId) {
        self.register_trigger_multiplier_source(id);
        self.register_activation_observer(id);
        self.register_zone_rule_card(self.state.objects[id as usize].card);
        self.register_static_base_stats(id);
        self.register_live_static_source(id);
        self.register_ability_source(id);
        self.register_hand_size_source(id);
        if self.state.objects[id as usize]
            .card
            .attachment_bonus()
            .is_some()
        {
            self.state.attachment_sources.push(id);
        }
        self.register_ability_target_source(id);
        let triggers = self.state.objects[id as usize].card.triggers();
        if triggers.iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::OtherCreatureCountersAdded { .. }
                    | TriggerCondition::OtherCreatureCounterPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYou { .. }
                    | TriggerCondition::CounterBatchPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                    | TriggerCondition::StateCounterAtLeast { .. }
            )
        }) {
            self.state.counter_sources.push(id);
        }
        if triggers.iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::AttachedCreatureBecomesTapped
                    | TriggerCondition::ControlledCreaturesBecomeTapped
                    | TriggerCondition::ControlledCreatureFirstTapOwnTurn
            )
        }) {
            self.state.tap_sources.push(id);
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
                    | TriggerCondition::FirstAttackWithTotalPowerAtLeast(_)
                    | TriggerCondition::AttackPlayer(_)
                    | TriggerCondition::ControlledCreatureAttacksAlone
                    | TriggerCondition::ControlledCreatureAttacks
                    | TriggerCondition::ControlledCreatureAttacksPlayerAlone
                    | TriggerCondition::OpponentCreatureBlocksSmall
                    | TriggerCondition::ControlledEquippedCreatureAttacks
            )
        }) {
            self.state.attack_sources.push(id);
        }
        if triggers
            .iter()
            .any(|ability| ability.condition.event() == TriggerCondition::ScryOrSurveil)
        {
            self.state.scry_sources.push(id);
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
                    | TriggerCondition::CreatureDamaged { .. }
                    | TriggerCondition::SelfDamaged
                    | TriggerCondition::SourceDealtCombatDamage
                    | TriggerCondition::AttachedDealtCombatDamage
                    | TriggerCondition::ControlledCreatureCombatDamageOwnTurn
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
                TriggerCondition::PermanentCardsEnterYourGraveyard
                    | TriggerCondition::CreatureCardsEnterYourGraveyard
                    | TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    | TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    | TriggerCondition::DamagedCreatureDies
                    | TriggerCondition::SelfDies
                    | TriggerCondition::SelfNontokenDies
                    | TriggerCondition::SelfCreatureDies
                    | TriggerCondition::SelfLeaves
                    | TriggerCondition::ReturnDeath(_)
                    | TriggerCondition::SelfEntersOrDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::OtherControlledPermanentWithSubtypeDies { .. }
                    | TriggerCondition::AttachedCreatureDies
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesWithMaximumStat { .. }
                    | TriggerCondition::CreatureDiesBatch { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::CreatureDiesAttackState { .. }
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::SacrificedByOpponent { .. }
            )
        }) {
            self.state.death_sources.push(id);
        }
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| {
                ability.condition.origin() != Some(TargetZone::Graveyard)
                    && matches!(
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
                        crate::cards::StaticAbilityFragment::ControlledInstantSorcerySplitSecond
                            | crate::cards::StaticAbilityFragment::ReplaceLifePaymentWithLibraryExile
                            | crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                            | crate::cards::StaticAbilityFragment::Storied
                            | crate::cards::StaticAbilityFragment::DoubleDrawExceptFirst
                            | crate::cards::StaticAbilityFragment::AdditionalTriggers { .. }
                            | crate::cards::StaticAbilityFragment::AttackControllerTax { .. }
                            | crate::cards::StaticAbilityFragment::OpponentNoncreatureSpellTax(_)
                            | crate::cards::StaticAbilityFragment::LandMana { .. }
                            | crate::cards::StaticAbilityFragment::ChosenBasicLandType
                            | crate::cards::StaticAbilityFragment::SpellImprovise(_)
                            | crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(_)
                            | crate::cards::StaticAbilityFragment::AttacksEachCombat
                            | crate::cards::StaticAbilityFragment::FlyingCombatRestriction
                            | crate::cards::StaticAbilityFragment::CannotBeBlockedBySubtype(_)
                            | crate::cards::StaticAbilityFragment::CannotBeBlockedIfSourcePowerAtMost(_)
                            | crate::cards::StaticAbilityFragment::CannotBeBlockedByCreaturesWithLessPower
                            | crate::cards::StaticAbilityFragment::ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(_)
                            | crate::cards::StaticAbilityFragment::ConditionalWard { .. }
                            | crate::cards::StaticAbilityFragment::HarnessedTrigger(..)
                            | crate::cards::StaticAbilityFragment::PlayerHexproofIfSourceCounter { .. }
                            | crate::cards::StaticAbilityFragment::PlayerHexproof
                            | crate::cards::StaticAbilityFragment::ActivateControlledCreaturesAsHaste
                            | crate::cards::StaticAbilityFragment::CantLoseGameAndOpponentsCantWin
                            | crate::cards::StaticAbilityFragment::ControlledSpellsUncounterable(_)
                            | crate::cards::StaticAbilityFragment::ControlledSpellsCasualty { .. }
                            | crate::cards::StaticAbilityFragment::SelfUntap(_)
                            | crate::cards::StaticAbilityFragment::CounterProhibition(_)
                            | crate::cards::StaticAbilityFragment::PlayerCounterProhibition { .. }
                            | crate::cards::StaticAbilityFragment::LifeGainAdditional(_)
                            | crate::cards::StaticAbilityFragment::NoMaximumHandSize
                            | crate::cards::StaticAbilityFragment::MaximumHandSize(_)
                            | crate::cards::StaticAbilityFragment::AdditionalLandPlays(_)
                            | crate::cards::StaticAbilityFragment::ConditionalAdditionalLandPlays { .. }
                            | crate::cards::StaticAbilityFragment::CantGainLife
                            | crate::cards::StaticAbilityFragment::ChosenModifier(..)
                            | crate::cards::StaticAbilityFragment::ChosenTypeKeywords(..)
                            | crate::cards::StaticAbilityFragment::SourceCounterBonus { .. }
                            | crate::cards::StaticAbilityFragment::SourceGraveyardBonus { .. }
                            | crate::cards::StaticAbilityFragment::ConditionalCreatureModifier { .. }
                            | crate::cards::StaticAbilityFragment::DeathBaronLord
                            | crate::cards::StaticAbilityFragment::ControlledPermanentKeywords { .. }
                            | crate::cards::StaticAbilityFragment::ControlledSubtypeKeywords { .. }
                            | crate::cards::StaticAbilityFragment::PlusOnePlacedKeywords { .. }
                            | crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(..)
                            | crate::cards::StaticAbilityFragment::CounterKeywordOther(..)
                    )
                })
            })
            || matches!(
                self.state.objects[id as usize].card.zone_replacement(),
                Some(
                    crate::cards::ZoneReplacementRule::ExileInstantSorceryCards
                        | crate::cards::ZoneReplacementRule::ExileAllCardsAndTokens
                        | crate::cards::ZoneReplacementRule::ExileOpposingCreatureDeath { .. }
                        | crate::cards::ZoneReplacementRule::ExileOpposingCreatureDeathPlain
                )
            )
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
            || self.state.objects[id as usize].card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::FirstEquipZeroCost { .. } | crate::cards::StaticAbilityFragment::EquipTargetReduction(_))))
            || zone_replacements::card_provides_prevention(self.state.objects[id as usize].card)
            || self.state.objects[id as usize].card.graveyard_activation_subtype().is_some()
            || self.state.objects[id as usize].card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::ControlledPlaneswalkerActivation(_) | crate::cards::StaticAbilityFragment::ControlledPlaneswalkerAttackLimit)))
            || self.state.objects[id as usize].card.static_ability_groups().iter().any(|group|
                group.fragments.iter().any(|fragment| matches!(fragment,
                    crate::cards::StaticAbilityFragment::ControlledAttackWithoutDefender
                    | crate::cards::StaticAbilityFragment::ControlledCombatDamageByGreaterToughness)))
            || self.state.objects[id as usize].card.has_creature_count_free_hand_cast()
            || self.state.objects[id as usize].card.has_free_hand_cast()
            || self.state.objects[id as usize].card.has_storm_grant()
            || self.state.objects[id as usize]
                .card
                .has_own_turn_free_hand_cast()
            || self.state.objects[id as usize]
                .card
                .has_cast_origin_abilities()
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
