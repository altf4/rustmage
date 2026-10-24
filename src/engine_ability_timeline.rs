//! Sparse, owned chronology for effects that must survive later ability removal.
use super::*;
use crate::cards::StaticAbilityFragment;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AbilitySourceTimestampView {
    pub source: ObjectRef,
    pub timestamp: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum KeywordEffectOrigin {
    Resolution { source: ObjectRef },
    Suspected,
}
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
impl Card {
    pub(super) fn has_layer6_provider(self) -> bool {
        self.role().is_some()
            || self.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::RemoveAttachedAbilities
                            | StaticAbilityFragment::ControlAttached
                            | StaticAbilityFragment::AttachedCharacteristics(..)
                            | StaticAbilityFragment::SourceCounterSubtype { .. }
                            | StaticAbilityFragment::SourceCounterKeywords { .. }
                            | StaticAbilityFragment::SourcePlusOnePlacedKeywords(..)
                            | StaticAbilityFragment::ChosenCreatureType
                            | StaticAbilityFragment::AttachedSubtype(..)
                            | StaticAbilityFragment::KeywordGrant(..)
                            | StaticAbilityFragment::AttachmentKeywords(..)
                            | StaticAbilityFragment::AttachedTurnKeywords(..)
                            | StaticAbilityFragment::AttachmentTrigger(..)
                            | StaticAbilityFragment::AttachmentActivation(..)
                            | StaticAbilityFragment::AttachmentMana(..)
                            | StaticAbilityFragment::AttachmentSacrificeMana(..)
                            | StaticAbilityFragment::LandMana { .. }
                            | StaticAbilityFragment::CounterKeyword(..)
                            | StaticAbilityFragment::CounterKeywordOwnTurn(..)
                            | StaticAbilityFragment::DeathBaronLord
                            | StaticAbilityFragment::LiveKeywords(..)
                    )
                })
            })
    }
}
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
    pub(super) fn keyword_subjects(&self, item: &SpellView, subject: Subject) -> Vec<CardId> {
        match subject {
            Subject::Entering => self.entering_subject(item).into_iter().collect(),
            Subject::Attached => self.attached_subject(item).into_iter().collect(),
            Subject::Source => (self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object)
                .then_some(item.card)
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
