//! Resolve ability availability against one borrowed actual or mana-overlay world.
//! Static candidates stay registered while blank, so leaving the removal source
//! restores them without a zone change or mutable derived cache.
use super::ability_order::{
    GrantedAbilityKey, GrantedManaAbility, LayerSixAction, LayerSixEffect, LayerSixGrant,
    LayerSixObject, LayerSixResult, apply_layer_six,
};
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{AbilityLayer, StaticAbilityFragment};

pub(super) struct EvaluatedAbilityGroup {
    pub source: ObjectRef,
    pub group: usize,
    pub first_layer: AbilityLayer,
    pub effect: Option<usize>,
}
pub(super) struct AbilityEvaluation {
    pub result: LayerSixResult,
    pub groups: Vec<EvaluatedAbilityGroup>,
}
impl AbilityEvaluation {
    pub fn group_available(&self, source: ObjectRef, group: usize) -> bool {
        let row = self
            .groups
            .iter()
            .find(|r| r.source == source && r.group == group)
            .expect("bound immutable static-effect group");
        row.first_layer < AbilityLayer::Ability
            || row.effect.is_some_and(|i| self.result.started[i])
            || (row.effect.is_none() && self.result.objects[source.card as usize].printed)
    }
}

fn timestamp<W: CharacteristicWorld + ?Sized>(world: &W, source: ObjectRef) -> u64 {
    world
        .roles()
        .and_then(|state| {
            state
                .ability_timestamps
                .iter()
                .find(|row| row.source == source)
                .map(|row| row.timestamp)
        })
        .expect("layer6 static provider has a shared entry/attachment timestamp")
}
fn grant_fragment<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    fragment: &StaticAbilityFragment,
    id: CardId,
    time: u64,
) -> Option<LayerSixGrant> {
    let provider = world.object(source.card);
    let target = world.object(id);
    if target.zone != Zone::Battlefield {
        return None;
    }
    let own = provider.controller == target.controller;
    let key = |ability| GrantedAbilityKey {
        source,
        ability,
        granting_card: provider.card,
        grantor_controller: provider.controller,
        timestamp: time,
    };
    let keywords = match *fragment {
        StaticAbilityFragment::AttachmentKeywords(value)
        | StaticAbilityFragment::AttachmentRule(value) => {
            if !world.attached(source, id) {
                return None;
            }
            value
        }
        StaticAbilityFragment::AttachedTurnKeywords(value) => {
            if provider.controller != world.game().state.active || !world.attached(source, id) {
                return None;
            }
            value
        }
        StaticAbilityFragment::AttachmentMana(production)
        | StaticAbilityFragment::AttachmentSacrificeMana(production) => {
            if !world.attachment_link(source, world.reference(id))
                || world.kinds(source.card) & 1 != 0
            {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![GrantedManaAbility {
                    source,
                    production,
                    sacrifice: matches!(
                        fragment,
                        StaticAbilityFragment::AttachmentSacrificeMana(_)
                    ),
                }],
            });
        }
        StaticAbilityFragment::LandMana {
            recipients,
            production,
        } => {
            if !world.land_mana_applies(source, id, recipients) {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![GrantedManaAbility {
                    source,
                    production,
                    sacrifice: false,
                }],
            });
        }
        StaticAbilityFragment::ConditionalWard {
            condition,
            filter,
            ability,
        } => {
            if !world.conditional_ward_applies(source, condition, filter, id) {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: vec![key(ability)],
                mana: Vec::new(),
            });
        }
        StaticAbilityFragment::HarnessedTrigger(ability) => {
            if id != source.card || !world.harnessed(source.card) {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: vec![key(ability)],
                mana: Vec::new(),
            });
        }
        StaticAbilityFragment::AttachmentTrigger(ability)
        | StaticAbilityFragment::AttachmentActivation(ability) => {
            if !world.attached(source, id) {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: vec![key(ability)],
                mana: Vec::new(),
            });
        }
        StaticAbilityFragment::ControlledPlaneswalkerActivation(ability) => {
            if !own || world.kinds(id) & 64 == 0 {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: vec![key(ability)],
                mana: Vec::new(),
            });
        }
        StaticAbilityFragment::ControlledPermanentKeywords { kinds, keywords } => {
            if !own || world.kinds(id) & kinds == 0 {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::ControlledSubtypeKeywords { subtype, keywords } => {
            if !own || !world.subtypes(id).has(world.definition(id), subtype) {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::DeathBaronLord => {
            if !world.death_baron_applies(source, id) {
                return None;
            }
            Keywords::DEATHTOUCH
        }
        StaticAbilityFragment::PlusOnePlacedKeywords { filter, keywords } => {
            if !world.placement_history_recipient(source, filter, id) {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::SourceConditionalKeywords {
            condition,
            keywords,
        } => {
            if world.reference(id) != source || world.value(condition, source.card) == 0 {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::SourcePlusOnePlacedKeywords(keywords) => {
            if id != source.card || !world.plus_one_placed_this_turn(source, provider.controller) {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::SourceCounterKeywords {
            kind,
            minimum,
            keywords,
        }
        | StaticAbilityFragment::SourceCounterSubtype {
            kind,
            minimum,
            keywords,
            ..
        } => {
            if id != source.card || world.counter_count(source.card, kind) < minimum {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::CounterKeywordOwnTurn(kind, value) => {
            if provider.controller != world.game().state.active
                || !own
                || world.kinds(id) & 1 == 0
                || world.counter_count(id, kind) == 0
            {
                return None;
            }
            value
        }
        StaticAbilityFragment::CounterKeywordOther(kind, value) => {
            if !own
                || world.reference(id) == source
                || world.kinds(id) & 1 == 0
                || world.counter_count(id, kind) == 0
            {
                return None;
            }
            value
        }
        StaticAbilityFragment::CounterKeyword(_, kind, value) => {
            if !own || world.kinds(id) & 1 == 0 || world.counter_count(id, kind) == 0 {
                return None;
            }
            value
        }
        StaticAbilityFragment::KeywordGrant(_, grant)
        | StaticAbilityFragment::StaticRuleGrant(_, grant) => {
            if grant
                .condition
                .is_some_and(|value| world.value(value, source.card) == 0)
                || (grant.requires_life_gain && !world.gained_life(provider.controller))
                || !grant.matches_characteristics(
                    target.card,
                    target.is_token(),
                    world.kinds(id),
                    world.subtypes(id),
                    own,
                    world.reference(id) == source,
                    world.attacking(id),
                    provider.controller == world.game().state.active,
                )
            {
                return None;
            }
            grant.keywords
        }
        StaticAbilityFragment::LiveKeywords(_, effect) => {
            if !world.live_applies(source, &effect, id) {
                return None;
            }
            effect.keywords
        }
        _ => return None,
    };
    Some(LayerSixGrant {
        recipient: id,
        keywords,
        abilities: Vec::new(),
        mana: Vec::new(),
    })
}

pub(super) fn evaluate_if_needed<W: CharacteristicWorld + ?Sized>(
    world: &W,
) -> Option<AbilityEvaluation> {
    (world.has_ability_removal()
        || world.static_sources().iter().any(|&id| {
            let source = world.object(id);
            source.zone == Zone::Battlefield
                && source.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| matches!(
                        fragment, StaticAbilityFragment::ControlledPlaneswalkerActivation(_)
                    ))
                })
        })
        // External attachment rules must participate even without ability removal.
        // They are deliberately absent from EquipmentSpec's granted abilities.
        || world.attachment_sources().iter().any(|&id| {
            let source = world.object(id);
            source.zone == Zone::Battlefield
                && source.attached_to.is_some()
                && source.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(fragment, StaticAbilityFragment::AttachmentRule(_))
                    })
                })
        })
        || world.roles().is_some_and(|r| {
            r.harnessed.iter().any(|reference| {
                world.reference(reference.card) == *reference
                    && world.object(reference.card).zone == Zone::Battlefield
                    && world
                        .object(reference.card)
                        .card
                        .static_ability_groups()
                        .iter()
                        .any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(fragment, StaticAbilityFragment::HarnessedTrigger(_))
                            })
                        })
            }) || r.source_linked_effects.iter().any(|row| {
                row.kind == SourceLinkedEffectKind::GrantHexproof && source_linked_effects::active(world, row)
            }) || r.keyword_effects.iter().any(|row| {
                (matches!(
                    world.object(row.recipient.card).zone,
                    Zone::Battlefield | Zone::Stack
                ) || matches!(row.origin, KeywordEffectOrigin::Counter { .. }))
                    && keyword_counters::applies(world, row)
                    && world.reference(row.recipient.card) == row.recipient
            }) || r.animations.iter().any(|row| {
                row.keywords != Keywords::NONE
                    && world.object(row.object.card).zone == Zone::Battlefield
                    && world.reference(row.object.card) == row.object
            })
        }))
    .then(|| evaluate_abilities(world))
}

pub(super) fn evaluate_abilities<W: CharacteristicWorld + ?Sized>(world: &W) -> AbilityEvaluation {
    let game = world.game();
    let mut effects = Vec::new();
    let mut groups = Vec::new();
    let objects = game
        .state
        .objects
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let object = world.object(i as CardId);
            LayerSixObject {
                printed: true,
                granted_keywords: Keywords::NONE,
                keywords: if object.zone == Zone::Stack {
                    game.effective_spell_card(i as CardId).keywords()
                } else {
                    world
                        .intrinsic_keywords(i as CardId)
                        .union(world.copied_keywords(i as CardId))
                },
                grants: Vec::new(),
                self_grants: Vec::new(),
                mana: Vec::new(),
            }
        })
        .collect();
    for (i, _) in game.state.objects.iter().enumerate() {
        let id = i as CardId;
        let object = world.object(id);
        if object.zone != Zone::Battlefield
            && !(object.zone == Zone::Command && object.card.is_emblem())
        {
            continue;
        }
        let source = world.reference(id);
        if world.kinds(id) & 2 != 0
            && object.card.static_ability_groups().iter().any(|group| {
                group
                    .fragments
                    .iter()
                    .any(|fragment| matches!(fragment, StaticAbilityFragment::ChosenBasicLandType))
            })
            && let Some(EntryChoiceValue::BasicLandType(kind)) = world.entry_value(source)
        {
            let mut production = [0; 6];
            production[kind.index()] = 1;
            effects.push(LayerSixEffect {
                provider: Some(id),
                started_earlier: true,
                timestamp: timestamp(world, source),
                recipients: vec![id],
                action: LayerSixAction::RemovePrintedAndGrant(vec![LayerSixGrant {
                    recipient: id,
                    keywords: Keywords::NONE,
                    abilities: Vec::new(),
                    mana: vec![GrantedManaAbility {
                        source,
                        production: ManaProduction::Fixed(production),
                        sacrifice: false,
                    }],
                }]),
            });
        }
        for (group, origin) in object.card.static_ability_groups().iter().enumerate() {
            let removes = origin
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::RemoveAttachedAbilities));
            let provides = origin.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::SourcePlusOnePlacedKeywords(..)
                        | StaticAbilityFragment::PlusOnePlacedKeywords { .. }
                        | StaticAbilityFragment::SourceConditionalKeywords { .. }
                        | StaticAbilityFragment::SourceCounterKeywords { .. }
                        | StaticAbilityFragment::SourceCounterSubtype { .. }
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
                        | StaticAbilityFragment::KeywordGrant(..)
                        | StaticAbilityFragment::ControlledPermanentKeywords { .. }
                        | StaticAbilityFragment::ControlledPlaneswalkerActivation(..)
                        | StaticAbilityFragment::ControlledSubtypeKeywords { .. }
                        | StaticAbilityFragment::LiveKeywords(..)
                )
            });
            let effect = if removes || provides {
                let time = timestamp(world, source);
                let recipients: Vec<_> = if removes {
                    object
                        .attached_to
                        .filter(|recipient| world.attachment_link(source, *recipient))
                        .map(|recipient| vec![recipient.card])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                let mut rows = Vec::new();
                if provides {
                    for (target, _) in game.state.objects.iter().enumerate() {
                        for fragment in origin.fragments {
                            if let Some(row) =
                                grant_fragment(world, source, fragment, target as CardId, time)
                            {
                                rows.push(row);
                            }
                        }
                    }
                }
                let action = if removes {
                    if provides {
                        LayerSixAction::RemoveAndGrant(rows)
                    } else {
                        LayerSixAction::Remove
                    }
                } else {
                    LayerSixAction::GrantEach(rows)
                };
                let index = effects.len();
                effects.push(LayerSixEffect {
                    provider: Some(id),
                    started_earlier: origin.first_layer < AbilityLayer::Ability,
                    timestamp: time,
                    recipients,
                    action,
                });
                Some(index)
            } else {
                None
            };
            groups.push(EvaluatedAbilityGroup {
                source,
                group,
                first_layer: origin.first_layer,
                effect,
            });
        }
    }
    if let Some(state) = world.roles() {
        for row in &state.resolved_ability_modifications {
            if return_ability_modification::active(world, row) {
                effects.push(LayerSixEffect {
                    provider: None,
                    started_earlier: false,
                    timestamp: row.timestamp,
                    recipients: vec![row.recipient.card],
                    action: LayerSixAction::RemoveAndGrant(vec![LayerSixGrant {
                        recipient: row.recipient.card,
                        keywords: row.keywords,
                        abilities: Vec::new(),
                        mana: Vec::new(),
                    }]),
                });
            }
        }
        for row in &state.source_linked_effects {
            if !source_linked_effects::active(world, row) {
                continue;
            }
            let action = match row.kind {
                SourceLinkedEffectKind::RemoveAbilities => LayerSixAction::Remove,
                SourceLinkedEffectKind::GrantHexproof => LayerSixAction::Grant {
                    keywords: Keywords::HEXPROOF,
                    abilities: Vec::new(),
                },
                _ => continue,
            };
            effects.push(LayerSixEffect {
                provider: None,
                started_earlier: false,
                timestamp: row.timestamp,
                recipients: vec![row.recipient.card],
                action,
            });
        }
        for row in &state.keyword_effects {
            let object = world.object(row.recipient.card);
            if (matches!(object.zone, Zone::Battlefield | Zone::Stack)
                || matches!(row.origin, KeywordEffectOrigin::Counter { .. }))
                && keyword_counters::applies(world, row)
                && world.reference(row.recipient.card) == row.recipient
                && row.expires_turn.is_none_or(|turn| turn == game.state.turn)
            {
                effects.push(LayerSixEffect {
                    provider: None,
                    started_earlier: false,
                    timestamp: row.timestamp,
                    recipients: vec![row.recipient.card],
                    action: LayerSixAction::Grant {
                        keywords: row.keywords,
                        abilities: Vec::new(),
                    },
                });
            }
        }
    }
    if let Some(state) = world.roles() {
        for row in &state.animations {
            if world.object(row.object.card).zone == Zone::Battlefield
                && world.reference(row.object.card) == row.object
                && row.keywords != Keywords::NONE
            {
                effects.push(LayerSixEffect {
                    provider: None,
                    started_earlier: true,
                    timestamp: row.timestamp,
                    recipients: vec![row.object.card],
                    action: LayerSixAction::Grant {
                        keywords: row.keywords,
                        abilities: Vec::new(),
                    },
                });
            }
        }
    }
    if let Some(grants) = &game.state.temporary_abilities {
        for grant in grants.iter() {
            let object = world.object(grant.recipient.card);
            if object.zone == Zone::Battlefield
                && world.reference(grant.recipient.card) == grant.recipient
                && (grant.permanent || grant.expires_turn == game.state.turn)
            {
                effects.push(LayerSixEffect {
                    provider: None,
                    started_earlier: false,
                    timestamp: grant.timestamp,
                    recipients: vec![grant.recipient.card],
                    action: LayerSixAction::Grant {
                        keywords: Keywords::NONE,
                        abilities: vec![GrantedAbilityKey {
                            source: grant.grantor,
                            granting_card: grant.granting_card,
                            grantor_controller: grant.grantor_controller,
                            ability: grant.ability,
                            timestamp: grant.timestamp,
                        }],
                    },
                });
            }
        }
    }
    if let Some(grants) = &game.state.self_granted_abilities {
        for grant in grants.iter() {
            if world.object(grant.recipient.card).zone == Zone::Battlefield
                && world.reference(grant.recipient.card) == grant.recipient
                && grant
                    .expires_turn
                    .is_none_or(|turn| turn == game.state.turn)
            {
                effects.push(LayerSixEffect {
                    provider: None,
                    started_earlier: false,
                    timestamp: grant.timestamp,
                    recipients: vec![grant.recipient.card],
                    action: LayerSixAction::GrantSelf(grant.timestamp),
                });
            }
        }
    }
    let mut evaluation = AbilityEvaluation {
        result: apply_layer_six(objects, &effects),
        groups,
    };
    apply_rule_effects(world, &mut evaluation);
    evaluation
}

/// CR613.11: external blocking restrictions affect rules, not abilities. They
/// apply after characteristic layers and cannot be erased from the recipient.
pub(super) fn apply_rule_effects<W: CharacteristicWorld + ?Sized>(
    world: &W,
    evaluation: &mut AbilityEvaluation,
) {
    let game = world.game();
    for (i, _) in game.state.objects.iter().enumerate() {
        let source = i as CardId;
        let object = world.object(source);
        if object.zone != Zone::Battlefield {
            continue;
        }
        let reference = world.reference(source);
        for (group, origin) in object.card.static_ability_groups().iter().enumerate() {
            if !evaluation.group_available(reference, group) {
                continue;
            }
            for fragment in origin.fragments {
                if !matches!(
                    fragment,
                    StaticAbilityFragment::AttachmentRule(..)
                        | StaticAbilityFragment::StaticRuleGrant(..)
                ) {
                    continue;
                }
                for (recipient, _) in game.state.objects.iter().enumerate() {
                    if let Some(row) =
                        grant_fragment(world, reference, fragment, recipient as CardId, 0)
                    {
                        evaluation.result.objects[recipient].keywords = evaluation.result.objects
                            [recipient]
                            .keywords
                            .union(row.keywords);
                    }
                }
            }
        }
    }
    if let Some(state) = world.roles() {
        for row in &state.keyword_effects {
            if matches!(
                world.object(row.recipient.card).zone,
                Zone::Battlefield | Zone::Stack
            ) && world.reference(row.recipient.card) == row.recipient
                && row.expires_turn.is_none_or(|turn| turn == game.state.turn)
            {
                let object = &mut evaluation.result.objects[row.recipient.card as usize];
                object.keywords = object.keywords.union(row.rules);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GrantedAbilityView {
    pub grantor: ObjectRef,
    pub granting_card: Card,
    pub ability: u16,
    pub grantor_controller: Player,
    pub timestamp: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AbilityAvailabilityView {
    pub printed: bool,
    pub grants: Vec<GrantedAbilityView>,
}
impl Game {
    pub(super) fn evaluated_abilities(&self) -> Option<AbilityEvaluation> {
        evaluate_if_needed(self)
    }
    pub(super) fn printed_abilities_available(&self, id: CardId) -> bool {
        CharacteristicWorld::printed_abilities_available(self, id)
    }
    pub(super) fn continuous_fragment_available(
        &self,
        source: CardId,
        matches: impl Fn(&StaticAbilityFragment) -> bool,
    ) -> bool {
        self.characteristic_query()
            .continuous_fragment_available(source, matches)
    }
}
impl ObjectView {
    pub(super) fn printed_abilities_available(&self) -> bool {
        self.abilities.as_ref().is_none_or(|value| value.printed)
    }
    pub(super) fn grant_available(
        &self,
        grantor: ObjectRef,
        granting_card: Card,
        ability: u16,
        timestamp: Option<u64>,
    ) -> bool {
        self.abilities.as_ref().is_none_or(|value| {
            value.grants.iter().any(|grant| {
                grant.grantor == grantor
                    && grant.granting_card == granting_card
                    && grant.ability == ability
                    && timestamp.is_none_or(|time| grant.timestamp == time)
            })
        })
    }
}
