//! Typed earlier-layer changes over borrowed actual or speculative object worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{
    AnimationCreatureTypes, AnimationDuration, AnimationSpec, AttachedCharacteristics,
    CharacteristicAddition, PermanentForm, SourceCreatureForm, StaticAbilityFragment,
};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AnimationView {
    pub object: ObjectRef,
    pub timestamp: u64,
    pub artifact: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub land: bool,
    #[serde(skip_serializing_if = "is_false")]
    /// Preserve existing card types; artifact may still be added independently.
    pub preserve_card_types: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub creature_only: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub legendary: bool,
    /// Replace all card types/subtypes with Artifact — Food, without removing abilities.
    #[serde(skip_serializing_if = "is_false")]
    pub food_only: bool,
    /// Set artifact card type, retaining instant/sorcery types per CR205.1a.
    #[serde(skip_serializing_if = "is_false")]
    pub artifact_noncreature: bool,
    /// The name and Equipment subtype of a resolved artifact-only form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_name: Option<&'static str>,
    /// Set enchantment card type, retaining instant/sorcery types per CR205.1a.
    #[serde(skip_serializing_if = "is_false")]
    pub enchantment_noncreature: bool,
    /// Card types added at this effect's timestamp.
    #[serde(skip_serializing_if = "is_zero_u16")]
    pub additional_card_types: u16,
    pub colors: Option<u8>,
    /// Colors added to the colors the object has at this effect's timestamp.
    #[serde(skip_serializing_if = "is_zero_u8")]
    pub additional_colors: u8,
    pub creature_types: Option<&'static [&'static str]>,
    pub all_creature_types: bool,
    /// This row adds named creature types instead of replacing them.
    #[serde(skip_serializing_if = "is_false")]
    pub additional_creature_types: bool,
    pub base: Option<(i128, i128)>,
    pub keywords: Keywords,
    pub expires_turn: Option<u32>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !value
}
fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum AnimationTypeMode {
    AddCreature,
    AddCreatureAndLand,
    PreserveCardTypes,
    ReplaceWithCreature,
    AddLegendary,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn aura_effects<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, &'static AttachedCharacteristics)> {
    let target = world.object(id);
    if target.zone != Zone::Battlefield {
        return Vec::new();
    }
    let mut effects = Vec::new();
    let Some(roles) = world.roles() else {
        return effects;
    };
    for &reference in &roles.characteristic_sources {
        let object = world.object(reference.card);
        if object.zone != Zone::Battlefield
            || world.reference(reference.card) != reference
            || object.attached_to != Some(world.reference(id))
        {
            continue;
        }
        for group in object.card.static_ability_groups() {
            if !entry_modes::group_enabled(world, reference, group) {
                continue;
            }
            for fragment in group.fragments {
                if let StaticAbilityFragment::AttachedCharacteristics(spec) = *fragment {
                    let timestamp = world
                        .roles()
                        .and_then(|r| r.ability_timestamps.iter().find(|r| r.source == reference))
                        .expect("registered characteristic provider")
                        .timestamp;
                    effects.push((timestamp, spec));
                }
            }
        }
    }
    effects
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn animations<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> impl Iterator<Item = &AnimationView> {
    let recipient = (world.object(id).zone == Zone::Battlefield).then(|| world.reference(id));
    world
        .roles()
        .into_iter()
        .flat_map(|r| &r.animations)
        .filter(move |row| Some(row.object) == recipient)
}

pub(super) fn has_equipment_form<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> bool {
    animations(world, id).any(|row| row.equipment_name.is_some())
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn grants_legendary<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> bool {
    world.roles().is_some_and(|r| !r.animations.is_empty())
        && animations(world, id).any(|row| row.legendary)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn source_type_forms<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> impl Iterator<Item = (&SourceLinkedEffectView, u16, SubtypeChange)> {
    let recipient = world.reference(id);
    world
        .roles()
        .into_iter()
        .flat_map(|roles| &roles.source_linked_effects)
        .filter_map(move |row| {
            let (kinds, change) = match row.kind {
                SourceLinkedEffectKind::LandUntilControllerNextTurn => (2, SubtypeChange::LandOnly),
                SourceLinkedEffectKind::Treasure | SourceLinkedEffectKind::ReturnedTreasure => {
                    (16, SubtypeChange::TreasureOnly)
                }
                _ => return None,
            };
            (row.recipient == recipient && source_linked_effects::active(world, row))
                .then_some((row, kinds, change))
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct SourceCreatureChange {
    timestamp: u64,
    kinds: u16,
    additive: bool,
    subtypes: &'static [&'static str],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn source_conditional_creature<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<SourceCreatureChange> {
    let object = world.object(id);
    if object.zone != Zone::Battlefield {
        return None;
    }
    let roles = world.roles()?;
    let (kinds, additive, subtypes) = object
        .card
        .static_ability_groups()
        .iter()
        .flat_map(|g| g.fragments)
        .find_map(|fragment| match *fragment {
            StaticAbilityFragment::SourceCounterCreature {
                own_turn,
                kind,
                minimum,
                form,
                subtypes,
                ..
            } if (!own_turn || object.controller == world.game().state.active)
                && world.counter_count(id, kind) >= minimum =>
            {
                let (kinds, additive) = match form {
                    SourceCreatureForm::Creature => (1, false),
                    SourceCreatureForm::ArtifactCreature => (17, true),
                    SourceCreatureForm::AddCreature => (1, true),
                };
                Some((kinds, additive, subtypes))
            }
            StaticAbilityFragment::SourceCelebrationCreatureForm { subtypes, .. }
                if world.celebrating(object.controller) =>
            {
                Some((0, true, subtypes))
            }
            StaticAbilityFragment::SourceOffTurnCreature
                if object.controller != world.game().state.active =>
            {
                Some((17, true, &[]))
            }
            _ => None,
        })?;
    let timestamp = roles
        .ability_timestamps
        .iter()
        .find(|row| row.source == world.reference(id))?
        .timestamp;
    Some(SourceCreatureChange {
        timestamp,
        kinds,
        additive,
        subtypes,
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn impending_type_timestamp<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Option<u64> {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || !object.cast_for_impending
        || object.card.impending_spec().is_none()
        || world.counter_count(id, CounterKind::Time) == 0
    {
        return None;
    }
    let source = world.reference(id);
    world
        .roles()?
        .ability_timestamps
        .iter()
        .find(|row| row.source == source)
        .map(|row| row.timestamp)
}

/// Layer-four changes to the presence of the land card type. Other card-type
/// additions do not affect existing land types and need no timeline entry.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn land_kind_events<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, bool)> {
    let mut events: Vec<_> = aura_effects(world, id)
        .into_iter()
        .filter_map(|(time, spec)| match spec.form {
            PermanentForm::AddCreature
            | PermanentForm::AddCreatureSubtypes
            | PermanentForm::Preserve
            | PermanentForm::PreserveWithColors => None,
            PermanentForm::AddCardTypes(types) => (types & 2 != 0).then_some((time, true)),
            form => Some((time, matches!(form, PermanentForm::Land))),
        })
        .collect();
    if let Some(change) = source_conditional_creature(world, id)
        && !change.additive
    {
        events.push((change.timestamp, false));
    }
    events.extend(animations(world, id).filter_map(|row| {
        if row.food_only
            || row.artifact_noncreature
            || row.enchantment_noncreature
            || row.creature_only
        {
            Some((row.timestamp, false))
        } else if row.additional_card_types & 2 != 0
            || row.land && (!row.preserve_card_types || row.artifact)
        {
            Some((row.timestamp, true))
        } else {
            None
        }
    }));
    events.extend(source_type_forms(world, id).map(|(row, kinds, _)| (row.timestamp, kinds == 2)));
    events.sort_by_key(|event| event.0);
    events
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn kinds<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    object: &Object,
) -> u16 {
    let mut result = original_kinds(world, id, object);
    if object.zone != Zone::Battlefield {
        return result;
    }
    if world
        .roles()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world.roles().is_none_or(|r| r.animations.is_empty())
        && source_conditional_creature(world, id).is_none_or(|change| change.kinds == 0)
        && impending_type_timestamp(world, id).is_none()
        && source_type_forms(world, id).next().is_none()
    {
        if world
            .game()
            .state
            .crew
            .as_ref()
            .is_some_and(|c| c.animated.binary_search(&world.reference(id)).is_ok())
        {
            result |= 17;
        }
        return result;
    }
    changed_kinds(world, id, result)
}
fn original_kinds<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId, object: &Object) -> u16 {
    // CR406.3a: permission to look at a face-down exiled card does not
    // give that card characteristics in exile.
    if object.zone == Zone::Exile && world.game().exiled_face_down(world.reference(id)) {
        return 0;
    }
    let mut result = if object.zone == Zone::Stack {
        world.game().effective_spell_card(id).kind_mask()
    } else {
        object.card.kind_mask()
    };
    let recipe = world.recipe(id);
    if recipe.exceptions != CopyExceptions::NONE || recipe.overrides.is_some() {
        result = recipe.kinds();
    }
    result
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
enum CardTypeChange {
    Add(u16),
    Set(u16),
    Remove(u16),
}

impl CardTypeChange {
    fn apply(self, kinds: u16) -> u16 {
        match self {
            Self::Add(mask) => kinds | mask,
            Self::Set(mask) => mask,
            Self::Remove(mask) => kinds & !mask,
        }
    }
}

// Keep uncommon projection/sort work out of ordinary type queries.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn kind_timeline<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    mut result: u16,
) -> (u16, Vec<(u64, CardTypeChange)>) {
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .filter_map(|(time, spec)| {
            let kinds = match spec.form {
                PermanentForm::Creature | PermanentForm::AddCreature => 1,
                PermanentForm::Land => 2,
                PermanentForm::FoodArtifact => 16,
                PermanentForm::AddCardTypes(types) => types,
                PermanentForm::Preserve
                | PermanentForm::PreserveWithColors
                | PermanentForm::AddCreatureSubtypes => return None,
            };
            let change = if matches!(
                spec.form,
                PermanentForm::AddCreature | PermanentForm::AddCardTypes(_)
            ) {
                CardTypeChange::Add(kinds)
            } else {
                CardTypeChange::Set(kinds)
            };
            Some((time, change))
        })
        .collect();
    if let Some(timestamp) = impending_type_timestamp(world, id) {
        changes.push((timestamp, CardTypeChange::Remove(1)));
    }
    if let Some(change) = source_conditional_creature(world, id)
        && change.kinds != 0
    {
        let effect = if change.additive {
            CardTypeChange::Add(change.kinds)
        } else {
            CardTypeChange::Set(change.kinds | (result & (4 | 8)))
        };
        changes.push((change.timestamp, effect));
    }
    changes.extend(
        source_type_forms(world, id)
            .map(|(row, kinds, _)| (row.timestamp, CardTypeChange::Set(kinds))),
    );
    for row in animations(world, id) {
        if row.food_only {
            changes.push((row.timestamp, CardTypeChange::Set(16)));
            continue;
        }
        if row.artifact_noncreature || row.enchantment_noncreature || row.creature_only {
            let kind = if row.creature_only {
                1
            } else if row.enchantment_noncreature {
                32
            } else {
                16
            };
            changes.push((
                row.timestamp,
                CardTypeChange::Set(kind | (result & (4 | 8))),
            ));
            continue;
        }
        let added = if row.preserve_card_types {
            if row.artifact { 16 } else { 0 }
        } else if row.artifact {
            17
        } else {
            1
        };
        let added = added | row.additional_card_types | if row.land { 2 } else { 0 };
        if added != 0 {
            changes.push((row.timestamp, CardTypeChange::Add(added)));
        }
    }
    if world
        .game()
        .state
        .crew
        .as_ref()
        .is_some_and(|c| c.animated.binary_search(&world.reference(id)).is_ok())
        && !animations(world, id).any(|r| r.artifact && !r.preserve_card_types)
    {
        result |= 17;
    }
    changes.sort_by_key(|c| c.0);
    if let Some(timestamp) = other_creatures_food_timestamp(world, id)
        && let Some(creature_since) = creature_scope_timestamp(result, &changes)
    {
        let timestamp = timestamp.max(creature_since);
        let after = changes.partition_point(|change| change.0 <= timestamp);
        changes.insert(after, (timestamp, CardTypeChange::Add(16)));
    }
    (result, changes)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_kinds<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId, result: u16) -> u16 {
    let (initial, changes) = kind_timeline(world, id, result);
    changes
        .into_iter()
        .fold(initial, |kinds, (_, change)| change.apply(kinds))
}
/// Latest layer-four provider applying to other creatures. Its effect began
/// before layer six and still applies if the provider subsequently loses abilities.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn other_creatures_food_timestamp<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<u64> {
    let roles = world.roles()?;
    roles
        .characteristic_sources
        .iter()
        .copied()
        .filter(|source| {
            let provider = world.object(source.card);
            source.card != id
                && provider.zone == Zone::Battlefield
                && world.reference(source.card) == *source
                && provider.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(fragment, StaticAbilityFragment::OtherCreaturesFood(_))
                    }) && entry_modes::group_enabled(world, *source, group)
                })
        })
        .filter_map(|source| {
            roles
                .ability_timestamps
                .iter()
                .find(|row| row.source == source)
                .map(|row| row.timestamp)
        })
        .max()
}

/// Fast entry paths have no entrant-specific layer-four changes. The existing
/// creature-to-Food effects still apply when determining entry replacements.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn fast_entry_kinds<W: CharacteristicWorld + ?Sized>(
    world: &W,
    entrant: CardId,
    kinds: u16,
) -> u16 {
    if kinds & (1 | 16) == 1 && other_creatures_food_timestamp(world, entrant).is_some() {
        kinds | 16
    } else {
        kinds
    }
}

/// A creature-only type or subtype grant depends on the effect that made its
/// recipient a creature. If the recipient is still a creature, return the
/// start of that uninterrupted creature scope (zero for intrinsic creatures).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn creature_scope_timestamp(mut kinds: u16, changes: &[(u64, CardTypeChange)]) -> Option<u64> {
    let mut latest = 0;
    for &(timestamp, change) in changes {
        let was_creature = kinds & 1 != 0;
        kinds = change.apply(kinds);
        if !was_creature && kinds & 1 != 0 {
            latest = timestamp;
        }
    }
    (kinds & 1 != 0).then_some(latest)
}

/// A creature-only subtype grant depends on an effect which first animates its
/// recipient. Its type addition follows that effect even with an older source.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn creature_addition_timestamp<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<u64> {
    let object = world.object(id);
    let (result, changes) = kind_timeline(world, id, original_kinds(world, id, object));
    creature_scope_timestamp(result, &changes).filter(|&timestamp| timestamp != 0)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn name<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> &'static str {
    let base = || rooms::battlefield_name(world, id).unwrap_or_else(|| world.copiable_name(id));
    match world.object(id).zone {
        Zone::Stack => world.game().effective_spell_card(id).name(),
        Zone::Battlefield => battlefield_name_change(world, id).unwrap_or_else(base),
        _ => base(),
    }
}

/// The names a battlefield object actually has, using separate halves for Rooms.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn battlefield_names<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> [&'static str; 2] {
    let Some(room) = world.definition(id).room() else {
        return [name(world, id), ""];
    };
    if let Some(name) = battlefield_name_change(world, id)
        .or_else(|| world.recipe(id).overrides.and_then(|value| value.name))
    {
        return [name, ""];
    }
    let doors = rooms::doors(world, id).expect("battlefield Room");
    RoomDoor::ALL.map(|door| {
        if doors.is_open(door) {
            room.halves[door.index()].name()
        } else {
            ""
        }
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn battlefield_name_change<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<&'static str> {
    if world.roles().is_none_or(|r| {
        r.characteristic_sources.is_empty()
            && r.animations.iter().all(|row| row.equipment_name.is_none())
    }) {
        return None;
    }
    aura_effects(world, id)
        .into_iter()
        .filter_map(|(t, s)| s.name.map(|name| (t, name)))
        .chain(
            animations(world, id)
                .filter_map(|row| row.equipment_name.map(|name| (row.timestamp, name))),
        )
        .max_by_key(|e| e.0)
        .map(|(_, name)| name)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || world.roles().is_none_or(|r| {
            r.characteristic_sources.is_empty()
                && r.animations
                    .iter()
                    .all(|a| a.colors.is_none() && a.additional_colors == 0)
        })
    {
        return if object.zone == Zone::Stack {
            world.game().effective_spell_card(id).colors()
        } else {
            rooms::battlefield_colors(world, id)
                .unwrap_or_else(|| world.recipe(id).copiable_colors())
        };
    }
    changed_colors(world, id)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    let setter = aura_effects(world, id)
        .into_iter()
        .filter_map(|(timestamp, spec)| {
            if matches!(
                spec.form,
                PermanentForm::AddCreature | PermanentForm::AddCreatureSubtypes
            ) || (matches!(
                spec.form,
                PermanentForm::Preserve | PermanentForm::AddCardTypes(_)
            ) && spec.colors.is_empty())
            {
                return None;
            }
            Some((
                timestamp,
                spec.colors
                    .iter()
                    .fold(0, |mask, c| mask | (1 << c.index())),
            ))
        })
        .chain(chosen_aura_colors(world, id))
        .chain(
            animations(world, id)
                .filter_map(|row| row.colors.map(|colors| (row.timestamp, colors))),
        )
        .max_by_key(|row| row.0);
    let colors = setter.map_or_else(
        || {
            rooms::battlefield_colors(world, id)
                .unwrap_or_else(|| world.recipe(id).copiable_colors())
        },
        |(_, colors)| colors,
    );
    animations(world, id)
        .filter(|row| {
            row.additional_colors != 0 && setter.is_none_or(|(time, _)| row.timestamp > time)
        })
        .fold(colors, |colors, row| colors | row.additional_colors)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn base_changes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, (i128, i128))> {
    if world
        .roles()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world.roles().is_none_or(|r| r.animations.is_empty())
    {
        return Vec::new();
    }
    changed_base(world, id)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_base<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, (i128, i128))> {
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .filter_map(|(t, s)| s.base.map(|b| (t, b)))
        .collect();
    changes.extend(animations(world, id).filter_map(|r| r.base.map(|b| (r.timestamp, b))));
    changes
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum SubtypeChange {
    Named(&'static [&'static str], bool),
    Add(&'static [&'static str]),
    RemoveCreatureTypes,
    Chosen(crate::cards::CreatureType),
    Granted(crate::cards::CreatureType, Option<&'static str>),
    All,
    LandOnly,
    BasicLand(crate::cards::BasicLandType),
    AddBasicLand(crate::cards::BasicLandType),
    FoodOnly,
    FoodArtifact,
    AddFood,
    TreasureOnly,
    EquipmentOnly,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn subtype_changes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, SubtypeChange)> {
    if world
        .roles()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world.roles().is_none_or(|r| r.animations.is_empty())
        && source_conditional_creature(world, id).is_none()
        && impending_type_timestamp(world, id).is_none()
        && source_type_forms(world, id).next().is_none()
    {
        return Vec::new();
    }
    changed_subtypes(world, id)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_subtypes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, SubtypeChange)> {
    let mut card_type_history = None;
    let mut can_have_creature_subtypes = |timestamp| {
        // Check the card types at the subtype effect's timestamp and after
        // all card-type changes (CR 205.3d, 613.7).
        let (initial, history) = card_type_history.get_or_insert_with(|| {
            let (initial, changes) =
                kind_timeline(world, id, original_kinds(world, id, world.object(id)));
            let mut kinds = initial;
            let history: Vec<_> = changes
                .into_iter()
                .map(|(time, change)| {
                    kinds = change.apply(kinds);
                    (time, kinds)
                })
                .collect();
            (initial, history)
        });
        let at = history.partition_point(|&(time, _)| time <= timestamp);
        let at_form = history[..at].last().map_or(*initial, |&(_, kinds)| kinds);
        let final_kinds = history.last().map_or(*initial, |&(_, kinds)| kinds);
        at_form & (1 | 128) != 0 && final_kinds & (1 | 128) != 0
    };
    let mut dependent_attachments = Vec::new();
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .filter_map(|(time, s)| {
            let change = match s.form {
                PermanentForm::Creature => SubtypeChange::Named(s.creature_types, true),
                PermanentForm::Land => SubtypeChange::LandOnly,
                PermanentForm::FoodArtifact => SubtypeChange::FoodArtifact,
                PermanentForm::Preserve | PermanentForm::PreserveWithColors => {
                    SubtypeChange::Named(s.creature_types, false)
                }
                PermanentForm::AddCreature => SubtypeChange::Add(s.creature_types),
                PermanentForm::AddCardTypes(_) => {
                    if !can_have_creature_subtypes(time) {
                        return None;
                    }
                    SubtypeChange::Add(s.creature_types)
                }
                PermanentForm::AddCreatureSubtypes => {
                    // A live attachment grant depends on the effect making its
                    // recipient a creature, even when that effect is newer.
                    let time = time.max(creature_addition_timestamp(world, id).unwrap_or(0));
                    if can_have_creature_subtypes(time) {
                        dependent_attachments.push((time, SubtypeChange::Add(s.creature_types)));
                    }
                    return None;
                }
            };
            Some((time, change))
        })
        .collect();
    if let Some(change) = source_conditional_creature(world, id)
        && (!change.additive || !change.subtypes.is_empty())
        && (change.kinds != 0 || can_have_creature_subtypes(change.timestamp))
    {
        changes.push((
            change.timestamp,
            SubtypeChange::Named(change.subtypes, !change.additive),
        ));
    }
    if let Some(timestamp) = impending_type_timestamp(world, id)
        && !can_have_creature_subtypes(timestamp)
    {
        changes.push((timestamp, SubtypeChange::RemoveCreatureTypes));
    }
    changes.extend(source_type_forms(world, id).map(|(row, _, change)| (row.timestamp, change)));
    changes.extend(animations(world, id).filter_map(|row| {
        if row.food_only {
            return Some((row.timestamp, SubtypeChange::FoodOnly));
        }
        if row.equipment_name.is_some() {
            return Some((row.timestamp, SubtypeChange::EquipmentOnly));
        }
        // Artifact-only form filters correlated subtypes against final layer-four
        // card types in the shared subtype projection; later animation may restore them.
        if row.artifact_noncreature || row.enchantment_noncreature {
            return None;
        }
        let change = if row.all_creature_types {
            SubtypeChange::All
        } else {
            let types = row.creature_types?;
            if row.additional_creature_types {
                SubtypeChange::Add(types)
            } else {
                SubtypeChange::Named(types, false)
            }
        };
        if row.preserve_card_types && !can_have_creature_subtypes(row.timestamp) {
            return None;
        }
        Some((row.timestamp, change))
    }));
    // The timestamp sort is stable, so a dependent grant follows the
    // subtype change on the same enabling animation.
    changes.extend(dependent_attachments);
    if let Some(timestamp) = other_creatures_food_timestamp(world, id) {
        let (initial, timeline) =
            kind_timeline(world, id, original_kinds(world, id, world.object(id)));
        if let Some(creature_since) = creature_scope_timestamp(initial, &timeline) {
            changes.push((timestamp.max(creature_since), SubtypeChange::AddFood));
        }
    }
    changes
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn add_characteristics(
        &mut self,
        item: &SpellView,
        subject: Subject,
        addition: CharacteristicAddition,
    ) {
        assert!(
            addition
                .creature_types
                .iter()
                .all(|name| crate::cards::is_creature_type(name))
        );
        let recipients: Vec<_> = self
            .keyword_subjects(item, subject)
            .into_iter()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let spec = AnimationSpec {
            artifact: false,
            creature_types: if addition.creature_types.is_empty() {
                AnimationCreatureTypes::Unchanged
            } else {
                AnimationCreatureTypes::Additional(addition.creature_types)
            },
            base: None,
            keywords: Keywords::NONE,
            duration: addition.duration,
        };
        let turn = self.state.turn;
        self.state
            .roles
            .as_mut()
            .unwrap()
            .animations
            .extend(recipients.into_iter().map(|object| AnimationView {
                additional_card_types: addition.card_types,
                additional_colors: addition.colors & 31,
                ..animation_view(
                    object,
                    timestamp,
                    spec,
                    AnimationTypeMode::PreserveCardTypes,
                    None,
                    turn,
                )
            }));
    }
    pub(super) fn animate_source_if_noncreature(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        ability: u16,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
            || self.is_creature(source.id)
        {
            return;
        }
        self.animate_source(item, spec);
        let timestamp = self
            .state
            .roles
            .as_ref()
            .unwrap()
            .animations
            .last()
            .unwrap()
            .timestamp;
        self.grant_source_permanent_trigger(item, ability, timestamp);
    }
    pub(super) fn animate_source_with_ability_if_noncreature(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        kind: SelfGrantedAbilityKind,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
            || self.is_creature(source.id)
        {
            return;
        }
        self.animate_source(item, spec);
        let timestamp = self
            .state
            .roles
            .as_ref()
            .unwrap()
            .animations
            .last()
            .unwrap()
            .timestamp;
        self.grant_self_ability_at(item, kind, timestamp);
    }
    pub(super) fn animate_source(&mut self, item: &SpellView, spec: AnimationSpec) {
        self.change_source_form(item, spec, AnimationTypeMode::AddCreature);
    }
    pub(super) fn become_legendary_equipment(
        &mut self,
        item: &SpellView,
        name: &'static str,
        equip: u16,
        power: i32,
        toughness: i32,
    ) {
        let Some(source) = item.source.map(|source| source.object) else {
            return;
        };
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let roles = self.state.roles.as_mut().unwrap();
        roles.animations.push(AnimationView {
            legendary: true,
            equipment_name: Some(name),
            ..artifact_noncreature_form(source, timestamp)
        });
        // At the shared timestamp the resolved removal precedes the acquired
        // abilities in layer six. Later grants can still apply to this object.
        roles
            .resolved_ability_modifications
            .push(ResolvedAbilityModificationView {
                recipient: source,
                source,
                keywords: Keywords::NONE,
                timestamp,
            });
        self.grant_recipient_ability(
            source,
            item.trigger_definition(),
            SelfGrantedAbilityKind::EquipAndBoost {
                equip,
                power,
                toughness,
            },
            timestamp,
            None,
        );
        if !self.state.attachment_sources.contains(&source.card) {
            self.state.attachment_sources.push(source.card);
        }
        self.refresh_copy_grant_availability();
    }
    pub(super) fn animate_source_colored(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        colors: u8,
    ) -> Option<AnimationView> {
        self.change_object_form_colored(
            item.source?.object,
            spec,
            AnimationTypeMode::AddCreature,
            Some(colors),
        )
    }
    pub(super) fn change_source_form(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        type_mode: AnimationTypeMode,
    ) {
        let Some(source) = item.source else {
            return;
        };
        self.change_object_form(source.object, spec, type_mode);
    }
    pub(super) fn add_artifact_to_targets(&mut self, item: &SpellView) {
        let spec = AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        };
        self.change_targets_form(item, spec, AnimationTypeMode::PreserveCardTypes);
    }
    pub(super) fn animate_subject(
        &mut self,
        item: &SpellView,
        subject: Subject,
        spec: AnimationSpec,
    ) {
        for id in self.keyword_subjects(item, subject) {
            self.change_object_form(self.object_ref(id), spec, AnimationTypeMode::AddCreature);
        }
    }
    pub(super) fn change_targets_form(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        type_mode: AnimationTypeMode,
    ) {
        for target in &item.targets {
            if self.same_target_object(*target)
                && let Some(object) = target.object
            {
                self.change_object_form(object, spec, type_mode);
            }
        }
    }
    pub(super) fn grant_legendary_creature_types(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
    ) {
        assert!(types.iter().all(|t| crate::cards::is_creature_type(t)));
        let spec = AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Additional(types),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        };
        for id in self.keyword_subjects(item, subject) {
            self.change_object_form(self.object_ref(id), spec, AnimationTypeMode::AddLegendary);
        }
    }

    pub(super) fn grant_all_creature_types(&mut self, item: &SpellView, subject: Subject) {
        let spec = AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::All,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        };
        let recipients = self.keyword_subjects(item, subject);
        for id in recipients {
            if self.on_battlefield(id) && self.is_creature(id) {
                self.change_object_form(
                    self.object_ref(id),
                    spec,
                    AnimationTypeMode::PreserveCardTypes,
                );
            }
        }
    }
    pub(super) fn animate_noncreature_targets(&mut self, item: &SpellView, spec: AnimationSpec) {
        for target in &item.targets {
            if self.same_target_object(*target)
                && let Some(object) = target.object
                && !self.is_creature(object.card)
            {
                self.change_object_form(object, spec, AnimationTypeMode::AddCreature);
            }
        }
    }
    pub(super) fn change_object_form(
        &mut self,
        object: ObjectRef,
        spec: AnimationSpec,
        type_mode: AnimationTypeMode,
    ) {
        let _ = self.change_object_form_colored(object, spec, type_mode, None);
    }
    fn change_object_form_colored(
        &mut self,
        object: ObjectRef,
        spec: AnimationSpec,
        type_mode: AnimationTypeMode,
        colors: Option<u8>,
    ) -> Option<AnimationView> {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return None;
        }
        let timestamp = self.next_role_timestamp();
        let row = animation_view(object, timestamp, spec, type_mode, colors, self.state.turn);
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(row);
        Some(row)
    }
    pub(super) fn choose_source_color_and_hexproof(&mut self, color: crate::cards::ChoiceColor) {
        let Some(source) = self.state.resolving.as_ref().and_then(|r| r.item.source) else {
            return;
        };
        let _ = self.change_object_form_colored(
            source.object,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::hexproof_from_color(color),
                duration: AnimationDuration::EndOfTurn,
            },
            AnimationTypeMode::PreserveCardTypes,
            Some(1 << color.index()),
        );
    }
    pub(super) fn set_creature_form(
        &mut self,
        item: &SpellView,
        types: &'static [&'static str],
        colors: u8,
        base: AmountValue,
    ) {
        let value =
            i128::try_from(self.item_amount(base, item)).capacity("creature form base exhausted");
        let spec = AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(types),
            base: Some((value, value)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        };
        for id in self.keyword_subjects(item, Subject::Targets) {
            let _ = self.change_object_form_colored(
                self.object_ref(id),
                spec,
                AnimationTypeMode::PreserveCardTypes,
                Some(colors),
            );
        }
    }
    pub(super) fn forget_animations(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.animations.retain(|r| r.object != object);
        }
    }
    pub(super) fn expire_animations(&mut self) -> bool {
        let Some(state) = &mut self.state.roles else {
            return false;
        };
        let before = state.animations.len();
        state.animations.retain(|r| r.expires_turn.is_none());
        state.animations.len() != before
    }
}

/// Build the same form in committed and projected characteristic worlds.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn animation_view(
    object: ObjectRef,
    timestamp: u64,
    spec: AnimationSpec,
    type_mode: AnimationTypeMode,
    colors: Option<u8>,
    turn: u32,
) -> AnimationView {
    AnimationView {
        object,
        timestamp,
        artifact: spec.artifact,
        land: matches!(type_mode, AnimationTypeMode::AddCreatureAndLand),
        preserve_card_types: matches!(
            type_mode,
            AnimationTypeMode::PreserveCardTypes | AnimationTypeMode::AddLegendary
        ),
        creature_only: matches!(type_mode, AnimationTypeMode::ReplaceWithCreature),
        legendary: matches!(type_mode, AnimationTypeMode::AddLegendary),
        food_only: false,
        artifact_noncreature: false,
        equipment_name: None,
        enchantment_noncreature: false,
        additional_card_types: 0,
        colors,
        additional_colors: 0,
        creature_types: match spec.creature_types {
            AnimationCreatureTypes::Named(t) | AnimationCreatureTypes::Additional(t) => Some(t),
            _ => None,
        },
        all_creature_types: matches!(spec.creature_types, AnimationCreatureTypes::All),
        additional_creature_types: matches!(
            spec.creature_types,
            AnimationCreatureTypes::Additional(_)
        ),
        base: spec.base,
        keywords: spec.keywords,
        expires_turn: match spec.duration {
            AnimationDuration::EndOfTurn => Some(turn),
            AnimationDuration::Permanent => None,
        },
    }
}

/// Same immutable form record for actual and speculative entry worlds.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn food_form(object: ObjectRef, timestamp: u64) -> AnimationView {
    AnimationView {
        object,
        timestamp,
        artifact: true,
        land: false,
        preserve_card_types: false,
        creature_only: false,
        legendary: false,
        food_only: true,
        artifact_noncreature: false,
        equipment_name: None,
        enchantment_noncreature: false,
        additional_card_types: 0,
        colors: None,
        additional_colors: 0,
        creature_types: None,
        all_creature_types: false,
        additional_creature_types: false,
        base: None,
        keywords: Keywords::NONE,
        expires_turn: None,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn apply_food_return(&mut self, recipient: ObjectRef, definition: Card) {
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .get_or_insert_default()
            .animations
            .push(food_form(recipient, timestamp));
        self.grant_recipient_ability(
            recipient,
            definition,
            SelfGrantedAbilityKind::Activated { ability: 0 },
            timestamp,
            None,
        );
    }
    pub(super) fn return_opponents_battlefield_graveyard_as_food(&mut self, item: &SpellView) {
        let moves = self
            .state
            .battlefield_graveyard_entries
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter_map(|&r| {
                let o = self.object_view(r.card);
                if o.reference != r
                    || o.zone != Zone::Graveyard
                    || o.owner == item.controller
                    || o.is_token()
                    || o.kinds & 1 == 0
                {
                    return None;
                }
                let mut movement = self.entry_move(r.card, Zone::Battlefield, false);
                movement.controller = item.controller;
                movement.food_return = Some(item.trigger_definition());
                Some(movement)
            })
            .collect();
        self.stage_entries(moves, None);
    }
}

/// CR611.2e form, shared by actual entry and speculative funding.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn artifact_noncreature_form(object: ObjectRef, timestamp: u64) -> AnimationView {
    AnimationView {
        food_only: false,
        artifact_noncreature: true,
        enchantment_noncreature: false,
        preserve_card_types: false,
        ..food_form(object, timestamp)
    }
}

/// A returned noncreature artifact enters as a flying Robot.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn flying_robot_form(object: ObjectRef, timestamp: u64) -> AnimationView {
    AnimationView {
        food_only: false,
        artifact: true,
        artifact_noncreature: false,
        preserve_card_types: false,
        creature_types: Some(&["Robot"]),
        base: Some((3, 3)),
        keywords: Keywords::FLYING,
        ..food_form(object, timestamp)
    }
}

/// Source-independent CR611.2e enchantment-only form.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn enchantment_noncreature_form(object: ObjectRef, timestamp: u64) -> AnimationView {
    AnimationView {
        food_only: false,
        artifact_noncreature: false,
        enchantment_noncreature: true,
        preserve_card_types: false,
        ..food_form(object, timestamp)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl AnimationView {
    pub(super) fn color(
        object: ObjectRef,
        timestamp: u64,
        colors: u8,
        expires_turn: Option<u32>,
    ) -> Self {
        Self {
            object,
            timestamp,
            artifact: false,
            land: false,
            preserve_card_types: true,
            creature_only: false,
            legendary: false,
            food_only: false,
            artifact_noncreature: false,
            equipment_name: None,
            enchantment_noncreature: false,
            additional_card_types: 0,
            colors: Some(colors & 31),
            additional_colors: 0,
            creature_types: None,
            all_creature_types: false,
            additional_creature_types: false,
            base: None,
            keywords: Keywords::NONE,
            expires_turn,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn set_color_effects(
        &mut self,
        subjects: &[CardId],
        colors: u8,
        duration: AnimationDuration,
    ) {
        for &id in subjects {
            if !self.on_battlefield(id) {
                continue;
            }
            let reference = self.object_ref(id);
            let timestamp = self.next_role_timestamp();
            let expires = (duration == AnimationDuration::EndOfTurn).then_some(self.state.turn);
            self.state
                .roles
                .get_or_insert_default()
                .animations
                .push(AnimationView::color(reference, timestamp, colors, expires));
        }
    }
    pub(super) fn apply_mana_color_change(
        &mut self,
        recipe: ManaProduction,
        source: ObjectRef,
        production: Mana,
    ) {
        if matches!(recipe.base(), ManaProduction::AnyColorChangingSource(_))
            && self.object_ref(source.card) == source
            && self.on_battlefield(source.card)
        {
            let colors = production
                .iter()
                .enumerate()
                .take(5)
                .fold(0, |mask, (i, n)| mask | if *n > 0 { 1 << i } else { 0 });
            if colors != 0 {
                self.record_cast_mana_recolor(colors);
                self.set_color_effects(&[source.card], colors, AnimationDuration::EndOfTurn);
            }
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn chosen_aura_colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Vec<(u64, u8)> {
    let Some(roles) = world.roles() else {
        return Vec::new();
    };
    if world.object(id).zone != Zone::Battlefield {
        return Vec::new();
    }
    roles
        .characteristic_sources
        .iter()
        .filter_map(|&reference| {
            let provider = world.object(reference.card);
            if provider.zone != Zone::Battlefield
                || world.reference(reference.card) != reference
                || provider.attached_to != Some(world.reference(id))
                || !provider.card.static_ability_groups().iter().any(|g| {
                    g.fragments
                        .iter()
                        .any(|f| matches!(f, StaticAbilityFragment::AttachedChosenColor))
                })
            {
                return None;
            }
            let EntryChoiceValue::Color(color) = world.entry_value(reference)? else {
                return None;
            };
            let timestamp = roles
                .ability_timestamps
                .iter()
                .find(|r| r.source == reference)?
                .timestamp;
            Some((timestamp, 1 << color.index()))
        })
        .collect()
}
