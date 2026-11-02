//! Sparse, owned chronology for effects that must survive later ability removal.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AbilitySourceTimestampView {
    pub source: ObjectRef,
    pub timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum KeywordEffectOrigin {
    Resolution {
        source: ObjectRef,
    },
    /// Source-independent loss of exactly the recorded keyword mask until cleanup.
    ResolutionRemoval {
        source: ObjectRef,
    },
    ResolutionUntilNextTurn {
        source: ObjectRef,
        controller: Player,
    },
    Suspected,
    Counter {
        kind: CounterKind,
    },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct KeywordEffectView {
    pub recipient: ObjectRef,
    pub origin: KeywordEffectOrigin,
    pub keywords: Keywords,
    /// Resolution-created blocking restrictions are not granted abilities.
    pub rules: Keywords,
    pub timestamp: u64,
    pub expires_turn: Option<u32>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    pub(super) fn has_layer6_provider(self) -> bool {
        self.role().is_some()
            || self.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::BorrowGraveyardActivations(..)
                            | StaticAbilityFragment::RemoveAttachedAbilities
                            | StaticAbilityFragment::ControlAttached
                            | StaticAbilityFragment::AttachedCharacteristics(..)
                            | StaticAbilityFragment::SourceCounterSubtype { .. }
                            | StaticAbilityFragment::SourceCounterCreature { .. }
                            | StaticAbilityFragment::SourceGraveyardAllTypes { .. }
                            | StaticAbilityFragment::SourceConditionalKeywords { .. }
                            | StaticAbilityFragment::SourceCounterKeywords { .. }
                            | StaticAbilityFragment::SourcePlusOnePlacedKeywords(..)
                            | StaticAbilityFragment::PlusOnePlacedKeywords { .. }
                            | StaticAbilityFragment::ChosenCreatureType
                            | StaticAbilityFragment::ChosenTypeKeywords(..)
                            | StaticAbilityFragment::ChosenBasicLandType
                            | StaticAbilityFragment::AttachedSubtype(..)
                            | StaticAbilityFragment::AttachedLegendary
                            | StaticAbilityFragment::KeywordGrant(..)
                            | StaticAbilityFragment::ControlledPermanentKeywords { .. }
                            | StaticAbilityFragment::SourceMaxSpeedActivation(..)
                            | StaticAbilityFragment::ControlledPlaneswalkerActivation(..)
                            | StaticAbilityFragment::ControlledPlaneswalkerAttackLimit
                            | StaticAbilityFragment::ControlledSubtypeKeywords { .. }
                            | StaticAbilityFragment::AttachmentKeywords(..)
                            | StaticAbilityFragment::AttachedTurnKeywords(..)
                            | StaticAbilityFragment::HarnessedTrigger(..)
                            | StaticAbilityFragment::ConditionalWard { .. }
                            | StaticAbilityFragment::AttachmentTrigger(..)
                            | StaticAbilityFragment::AttachmentActivation(..)
                            | StaticAbilityFragment::AttachmentMana(..)
                            | StaticAbilityFragment::AttachmentSacrificeMana(..)
                            | StaticAbilityFragment::LandMana { .. }
                            | StaticAbilityFragment::CounterKeyword(..)
                            | StaticAbilityFragment::CounterKeywordOther(..)
                            | StaticAbilityFragment::CounterKeywordOwnTurn(..)
                            | StaticAbilityFragment::DeathBaronLord
                            | StaticAbilityFragment::LiveKeywords(..)
                    )
                })
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn register_ability_source(&mut self, id: CardId) {
        if !self.state.objects[id as usize].card.has_layer6_provider() {
            return;
        }
        let source = self.object_ref(id);
        if self
            .state
            .roles
            .as_ref()
            .is_some_and(|r| r.ability_timestamps.iter().any(|row| row.source == source))
        {
            return;
        }
        let timestamp = self.static_source_timestamp(source);
        if self.state.objects[id as usize]
            .card
            .static_ability_groups()
            .iter()
            .any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(fragment, StaticAbilityFragment::RemoveAttachedAbilities)
                })
            })
        {
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .ability_removal_sources
                .push(source);
        }
        if self.state.objects[id as usize]
            .card
            .static_ability_groups()
            .iter()
            .any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(fragment, StaticAbilityFragment::AttachedCharacteristics(..))
                })
            })
        {
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .characteristic_sources
                .push(source);
        }
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .ability_timestamps
            .push(AbilitySourceTimestampView { source, timestamp });
    }
    pub(super) fn forget_ability_source(&mut self, reference: ObjectRef) {
        self.forget_source_linked_effects(reference);
        if let Some(r) = &mut self.state.roles {
            r.resolved_ability_modifications
                .retain(|row| row.recipient != reference);
        }
        if let Some(state) = &mut self.state.roles {
            state
                .ability_timestamps
                .retain(|row| row.source != reference);
            state
                .ability_removal_sources
                .retain(|&source| source != reference);
            state
                .characteristic_sources
                .retain(|&source| source != reference);
            state
                .keyword_effects
                .retain(|row| row.recipient != reference);
        }
    }
    /// A cast-spending effect follows only this spell's actual stack-to-battlefield move.
    pub(super) fn grant_stack_keywords(&mut self, spell: ObjectRef, keywords: Keywords) {
        assert_eq!(self.object_ref(spell.card), spell);
        assert_eq!(self.state.objects[spell.card as usize].zone, Zone::Stack);
        if keywords == Keywords::NONE {
            return;
        }
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .as_mut()
            .unwrap()
            .keyword_effects
            .push(KeywordEffectView {
                recipient: spell,
                origin: KeywordEffectOrigin::Resolution { source: spell },
                keywords,
                rules: Keywords::NONE,
                timestamp,
                expires_turn: Some(self.state.turn),
            });
    }
    pub(super) fn continue_stack_keywords(
        &mut self,
        before: ObjectRef,
        after: ObjectRef,
        destination: Zone,
    ) {
        if let Some(state) = &mut self.state.roles {
            if destination == Zone::Battlefield {
                for row in &mut state.keyword_effects {
                    if row.recipient == before {
                        row.recipient = after;
                    }
                }
            } else {
                state.keyword_effects.retain(|row| row.recipient != before);
            }
        }
    }
    pub(super) fn grant_keyword_effects(
        &mut self,
        ids: &[CardId],
        source: ObjectRef,
        keywords: Keywords,
    ) {
        if ids.is_empty() || keywords == Keywords::NONE {
            return;
        }
        // One instruction is one effect, including every simultaneous recipient.
        let timestamp = self.next_role_timestamp();
        for &id in ids {
            let recipient = self.object_ref(id);
            let object = &mut self.state.objects[id as usize];
            object.gained_keywords = object.gained_keywords.union(keywords);
            self.state
                .roles
                .as_mut()
                .unwrap()
                .keyword_effects
                .push(KeywordEffectView {
                    recipient,
                    origin: KeywordEffectOrigin::Resolution { source },
                    keywords: keywords.without(keywords.resolution_rules()),
                    rules: keywords.resolution_rules(),
                    timestamp,
                    expires_turn: Some(self.state.turn),
                });
        }
    }
    pub(super) fn grant_keywords_until_next_turn(
        &mut self,
        ids: &[CardId],
        source: ObjectRef,
        controller: Player,
        keywords: Keywords,
    ) {
        self.grant_keyword_effects(ids, source, keywords);
        if ids.is_empty() || keywords == Keywords::NONE {
            return;
        }
        let rows = &mut self.state.roles.as_mut().unwrap().keyword_effects;
        let start = rows.len() - ids.len();
        for row in &mut rows[start..] {
            row.origin = KeywordEffectOrigin::ResolutionUntilNextTurn { source, controller };
            row.expires_turn = None;
        }
    }
    pub(super) fn expire_next_turn_keywords(&mut self, player: Player) {
        if let Some(state) = &mut self.state.roles {
            state.keyword_effects.retain(|row| {
                !matches!(row.origin, KeywordEffectOrigin::ResolutionUntilNextTurn { controller, .. } if controller == player)
            });
        }
    }
    pub(super) fn keyword_subjects(&self, item: &SpellView, subject: Subject) -> Vec<CardId> {
        match subject {
            Subject::Entering => self.entering_subject(item).into_iter().collect(),
            Subject::Attached => self.attached_subject(item).into_iter().collect(),
            Subject::Source => item
                .source
                .filter(|source| {
                    self.on_battlefield(source.id) && self.object_ref(source.id) == source.object
                })
                .map(|source| source.id)
                .into_iter()
                .collect(),
            Subject::Targets => item
                .targets
                .iter()
                .filter(|bound| self.same_target_object(**bound))
                .filter_map(|bound| match bound.target {
                    Target::Card(id) => Some(id),
                    _ => None,
                })
                .collect(),
        }
    }
    pub(super) fn record_suspect_keywords(&mut self, recipient: ObjectRef) {
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .as_mut()
            .unwrap()
            .keyword_effects
            .push(KeywordEffectView {
                recipient,
                origin: KeywordEffectOrigin::Suspected,
                keywords: Keywords::MENACE.union(Keywords::CANT_BLOCK),
                rules: Keywords::NONE,
                timestamp,
                expires_turn: None,
            });
    }
    pub(super) fn forget_suspect_keywords(&mut self, recipient: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.keyword_effects.retain(|row| {
                row.recipient != recipient || row.origin != KeywordEffectOrigin::Suspected
            });
        }
    }
    pub(super) fn expire_keyword_effects(&mut self) {
        if let Some(state) = &mut self.state.roles {
            state
                .keyword_effects
                .retain(|row| row.expires_turn.is_none());
        }
    }
}
