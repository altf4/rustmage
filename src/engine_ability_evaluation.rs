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

fn timestamp(game: &Game, source: ObjectRef) -> u64 {
    game.state
        .roles
        .as_ref()
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
        StaticAbilityFragment::AttachmentMana(production) => {
            if !world.attachment_link(source, world.reference(id))
                || world.kinds(source.card) & 1 != 0
            {
                return None;
            }
            return Some(LayerSixGrant {
                recipient: id,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![GrantedManaAbility { source, production }],
            });
        }
        StaticAbilityFragment::AttachmentTrigger(ability) => {
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
        StaticAbilityFragment::DeathBaronLord => {
            if !world.death_baron_applies(source, id) {
                return None;
            }
            Keywords::DEATHTOUCH
        }
        StaticAbilityFragment::SourceCounterSubtype {
            kind,
            minimum,
            keywords,
            ..
        } => {
            if id != source.card || provider.counters.get(kind) < minimum {
                return None;
            }
            keywords
        }
        StaticAbilityFragment::CounterKeywordOwnTurn(kind, value) => {
            if provider.controller != world.game().state.active
                || !own
                || world.kinds(id) & 1 == 0
                || target.counters.get(kind) == 0
            {
                return None;
            }
            value
        }
        StaticAbilityFragment::CounterKeyword(_, kind, value) => {
            if !own || world.kinds(id) & 1 == 0 || target.counters.get(kind) == 0 {
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
    world
        .has_ability_removal()
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
                keywords: object.card.keywords(),
                grants: Vec::new(),
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
        for (group, origin) in object.card.static_ability_groups().iter().enumerate() {
            let removes = origin
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::RemoveAttachedAbilities));
            let provides = origin.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::SourceCounterSubtype { .. }
                        | StaticAbilityFragment::AttachmentKeywords(..)
                        | StaticAbilityFragment::AttachedTurnKeywords(..)
                        | StaticAbilityFragment::AttachmentTrigger(..)
                        | StaticAbilityFragment::AttachmentMana(..)
                        | StaticAbilityFragment::CounterKeyword(..)
                        | StaticAbilityFragment::CounterKeywordOwnTurn(..)
                        | StaticAbilityFragment::DeathBaronLord
                        | StaticAbilityFragment::KeywordGrant(..)
                        | StaticAbilityFragment::LiveKeywords(..)
                )
            });
            let effect = if removes || provides {
                assert!(
                    !(removes && provides),
                    "removal and independent grants require distinct continuous-effect recipes"
                );
                let time = timestamp(game, source);
                let recipients: Vec<_> = if removes {
                    object
                        .attached_to
                        .filter(|recipient| world.attachment_link(source, *recipient))
                        .map(|recipient| vec![recipient.card])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                let action = if removes {
                    LayerSixAction::Remove
                } else {
                    let mut rows = Vec::new();
                    for (target, _) in game.state.objects.iter().enumerate() {
                        for fragment in origin.fragments {
                            if let Some(row) =
                                grant_fragment(world, source, fragment, target as CardId, time)
                            {
                                rows.push(row);
                            }
                        }
                    }
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
    if let Some(state) = &game.state.roles {
        for row in &state.keyword_effects {
            let object = world.object(row.recipient.card);
            if object.zone == Zone::Battlefield
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
    if let Some(grants) = &game.state.temporary_abilities {
        for grant in grants.iter() {
            let object = world.object(grant.recipient.card);
            if object.zone == Zone::Battlefield
                && world.reference(grant.recipient.card) == grant.recipient
                && grant.expires_turn == game.state.turn
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
    if let Some(state) = &game.state.roles {
        for row in &state.keyword_effects {
            if world.object(row.recipient.card).zone == Zone::Battlefield
                && world.reference(row.recipient.card) == row.recipient
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
