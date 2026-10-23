//! Typed earlier-layer changes over borrowed actual or speculative object worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{
    AnimationCreatureTypes, AnimationDuration, AnimationSpec, AttachedCharacteristics,
    PermanentForm, StaticAbilityFragment,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AnimationView {
    pub object: ObjectRef,
    pub timestamp: u64,
    pub artifact: bool,
    pub creature_types: Option<&'static [&'static str]>,
    pub all_creature_types: bool,
    pub additive_creature_types: bool,
    pub base: Option<(i128, i128)>,
    pub keywords: Keywords,
    pub expires_turn: Option<u32>,
}
fn aura_effects<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, &'static AttachedCharacteristics)> {
    let target = world.object(id);
    if target.zone != Zone::Battlefield {
        return Vec::new();
    }
    let mut effects = Vec::new();
    let Some(roles) = world.game().state.roles.as_ref() else {
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
            for fragment in group.fragments {
                if let StaticAbilityFragment::AttachedCharacteristics(spec) = *fragment {
                    let timestamp = world
                        .game()
                        .state
                        .roles
                        .as_ref()
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
fn animations<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> impl Iterator<Item = &AnimationView> {
    let recipient = (world.object(id).zone == Zone::Battlefield).then(|| world.reference(id));
    world
        .game()
        .state
        .roles
        .as_ref()
        .into_iter()
        .flat_map(|r| &r.animations)
        .filter(move |row| Some(row.object) == recipient)
}
#[inline]
pub(super) fn kinds<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    object: &Object,
) -> u16 {
    let mut result = if object.zone == Zone::Stack {
        world.game().effective_spell_card(id).kind_mask()
    } else {
        object.card.kind_mask()
    };
    if object.zone != Zone::Battlefield {
        return result;
    }
    if world
        .game()
        .state
        .roles
        .as_ref()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world
            .game()
            .state
            .roles
            .as_ref()
            .is_none_or(|r| r.animations.is_empty())
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
// Keep uncommon projection/sort work out of ordinary type queries.
fn changed_kinds<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId, mut result: u16) -> u16 {
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .filter_map(|(time, spec)| {
            let mask = match spec.form {
                PermanentForm::Preserve => return None,
                PermanentForm::Creature => 1,
                PermanentForm::Land => 2,
            };
            Some((time, mask, false))
        })
        .collect();
    for row in animations(world, id) {
        changes.push((row.timestamp, if row.artifact { 17 } else { 1 }, true));
    }
    if world
        .game()
        .state
        .crew
        .as_ref()
        .is_some_and(|c| c.animated.binary_search(&world.reference(id)).is_ok())
        && !animations(world, id).any(|r| r.artifact)
    {
        result |= 17;
    }
    changes.sort_by_key(|c| c.0);
    for (_, mask, additive) in changes {
        if additive {
            result |= mask;
        } else {
            result = mask;
        }
    }
    result
}
#[inline]
pub(super) fn name<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> &'static str {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || world
            .game()
            .state
            .roles
            .as_ref()
            .is_none_or(|r| r.characteristic_sources.is_empty())
    {
        return if object.zone == Zone::Stack {
            world.game().effective_spell_card(id).name()
        } else {
            object.card.name()
        };
    }
    changed_name(world, id)
}
fn changed_name<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> &'static str {
    aura_effects(world, id)
        .into_iter()
        .filter_map(|(t, s)| s.name.map(|name| (t, name)))
        .max_by_key(|e| e.0)
        .map_or_else(|| world.object(id).card.name(), |e| e.1)
}
#[inline]
pub(super) fn colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || world.game().state.roles.as_ref().is_none_or(|roles| {
            roles.characteristic_sources.is_empty() && roles.color_effects.is_empty()
        })
    {
        return if object.zone == Zone::Stack {
            world.game().effective_spell_card(id).colors()
        } else {
            object.card.colors()
        };
    }
    changed_colors(world, id)
}
fn changed_colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    let attached = aura_effects(world, id)
        .into_iter()
        .map(|(timestamp, spec)| {
            (
                timestamp,
                spec.colors
                    .iter()
                    .fold(0u8, |mask, color| mask | (1 << color.index())),
            )
        });
    let fixed = world
        .game()
        .state
        .roles
        .as_ref()
        .into_iter()
        .flat_map(|roles| &roles.color_effects)
        .filter(|row| row.object == world.reference(id))
        .map(|row| (row.timestamp, row.colors));
    let chosen = world
        .game()
        .state
        .roles
        .as_ref()
        .into_iter()
        .flat_map(|roles| &roles.characteristic_sources)
        .filter_map(|&source| {
            if !world.attachment_link(source, world.reference(id))
                || !world
                    .object(source.card)
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(fragment, StaticAbilityFragment::AttachedChosenColor)
                        })
                    })
            {
                return None;
            }
            let Some(EntryChoiceValue::Color(color)) = world.game().entry_value(source) else {
                return None;
            };
            let timestamp = world
                .game()
                .state
                .roles
                .as_ref()
                .unwrap()
                .ability_timestamps
                .iter()
                .find(|row| row.source == source)
                .expect("registered color provider")
                .timestamp;
            Some((timestamp, 1 << color.index()))
        });
    attached
        .chain(chosen)
        .chain(fixed)
        .max_by_key(|row| row.0)
        .map_or_else(|| world.object(id).card.colors(), |row| row.1)
}
#[inline]
pub(super) fn base_changes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, (i128, i128))> {
    if world
        .game()
        .state
        .roles
        .as_ref()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world
            .game()
            .state
            .roles
            .as_ref()
            .is_none_or(|r| r.animations.is_empty())
    {
        return Vec::new();
    }
    changed_base(world, id)
}
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
#[derive(Clone, Copy)]
pub(super) enum SubtypeChange {
    Named(&'static [&'static str], bool),
    Add(&'static [&'static str]),
    All,
    LandOnly,
}
#[inline]
pub(super) fn subtype_changes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, SubtypeChange)> {
    if world
        .game()
        .state
        .roles
        .as_ref()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world
            .game()
            .state
            .roles
            .as_ref()
            .is_none_or(|r| r.animations.is_empty())
    {
        return Vec::new();
    }
    changed_subtypes(world, id)
}
fn changed_subtypes<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<(u64, SubtypeChange)> {
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .map(|(time, s)| {
            (
                time,
                match s.form {
                    PermanentForm::Preserve => SubtypeChange::Named(s.creature_types, false),
                    PermanentForm::Creature => SubtypeChange::Named(s.creature_types, true),
                    PermanentForm::Land => SubtypeChange::LandOnly,
                },
            )
        })
        .collect();
    changes.extend(animations(world, id).filter_map(|row| {
        if row.all_creature_types {
            Some((row.timestamp, SubtypeChange::All))
        } else {
            row.creature_types.map(|t| {
                (
                    row.timestamp,
                    if row.additive_creature_types {
                        SubtypeChange::Add(t)
                    } else {
                        SubtypeChange::Named(t, false)
                    },
                )
            })
        }
    }));
    changes
}
impl Game {
    pub(super) fn animate_source(&mut self, item: &SpellView, spec: AnimationSpec) {
        self.animate_exact(item.object, spec);
    }
    pub(super) fn animate_targets_if_noncreature(&mut self, item: &SpellView, spec: AnimationSpec) {
        for bound in &item.targets {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
                && !self.is_creature(id)
            {
                self.animate_exact(self.object_ref(id), spec);
            }
        }
    }
    fn animate_exact(&mut self, object: ObjectRef, spec: AnimationSpec) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let row = AnimationView {
            object,
            timestamp,
            artifact: spec.artifact,
            creature_types: match spec.creature_types {
                AnimationCreatureTypes::Named(t) | AnimationCreatureTypes::AddNamed(t) => Some(t),
                _ => None,
            },
            all_creature_types: matches!(spec.creature_types, AnimationCreatureTypes::All),
            additive_creature_types: matches!(
                spec.creature_types,
                AnimationCreatureTypes::AddNamed(_)
            ),
            base: spec.base,
            keywords: spec.keywords,
            expires_turn: match spec.duration {
                AnimationDuration::EndOfTurn => Some(self.state.turn),
                AnimationDuration::Permanent => None,
            },
        };
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(row);
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
