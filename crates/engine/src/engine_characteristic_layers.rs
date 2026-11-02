//! Typed earlier-layer changes over borrowed actual or speculative object worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{
    AnimationCreatureTypes, AnimationDuration, AnimationSpec, AttachedCharacteristics,
    PermanentForm, StaticAbilityFragment,
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
    /// Replace all card types/subtypes with Artifact — Food, without removing abilities.
    #[serde(skip_serializing_if = "is_false")]
    pub food_only: bool,
    /// Set artifact card type, retaining instant/sorcery types per CR205.1a.
    #[serde(skip_serializing_if = "is_false")]
    pub artifact_noncreature: bool,
    /// Set enchantment card type, retaining instant/sorcery types per CR205.1a.
    #[serde(skip_serializing_if = "is_false")]
    pub enchantment_noncreature: bool,
    pub colors: Option<u8>,
    pub creature_types: Option<&'static [&'static str]>,
    pub all_creature_types: bool,
    pub base: Option<(i128, i128)>,
    pub keywords: Keywords,
    pub expires_turn: Option<u32>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !value
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum AnimationTypeMode {
    AddCreature,
    AddCreatureAndLand,
    PreserveCardTypes,
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn land_forms<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> impl Iterator<Item = &SourceLinkedEffectView> {
    let recipient = world.reference(id);
    world
        .roles()
        .into_iter()
        .flat_map(|roles| &roles.source_linked_effects)
        .filter(move |row| {
            row.kind == SourceLinkedEffectKind::LandUntilControllerNextTurn
                && row.recipient == recipient
                && source_linked_effects::active(world, row)
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn source_counter_turn_creature<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Option<(u64, &'static [&'static str])> {
    let object = world.object(id);
    if object.zone != Zone::Battlefield {
        return None;
    }
    let types = object
        .card
        .static_ability_groups()
        .iter()
        .flat_map(|g| g.fragments)
        .find_map(|fragment| match *fragment {
            StaticAbilityFragment::SourceCounterCreature {
                own_turn,
                kind,
                minimum,
                subtypes,
                ..
            } if (!own_turn || object.controller == world.game().state.active)
                && world.counter_count(id, kind) >= minimum =>
            {
                Some(subtypes)
            }
            _ => None,
        })?;
    let timestamp = world
        .roles()?
        .ability_timestamps
        .iter()
        .find(|row| row.source == world.reference(id))?
        .timestamp;
    Some((timestamp, types))
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
            PermanentForm::AddCreature => None,
            form => Some((time, matches!(form, PermanentForm::Land))),
        })
        .collect();
    events.extend(animations(world, id).filter_map(|row| {
        if row.food_only || row.artifact_noncreature || row.enchantment_noncreature {
            Some((row.timestamp, false))
        } else if row.land && (!row.preserve_card_types || row.artifact) {
            Some((row.timestamp, true))
        } else {
            None
        }
    }));
    events.extend(land_forms(world, id).map(|row| (row.timestamp, true)));
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
    let mut result = if object.zone == Zone::Stack {
        world.game().effective_spell_card(id).kind_mask()
    } else {
        object.card.kind_mask()
    };
    let recipe = world.recipe(id);
    if recipe.exceptions != CopyExceptions::NONE || recipe.overrides.is_some() {
        result = recipe.kinds();
    }
    if object.zone != Zone::Battlefield {
        return result;
    }
    if world
        .roles()
        .is_none_or(|r| r.characteristic_sources.is_empty())
        && world.roles().is_none_or(|r| r.animations.is_empty())
        && source_counter_turn_creature(world, id).is_none()
        && land_forms(world, id).next().is_none()
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_kinds<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId, mut result: u16) -> u16 {
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .map(|(time, spec)| {
            (
                time,
                match spec.form {
                    PermanentForm::Creature => 1,
                    PermanentForm::Land => 2,
                    PermanentForm::AddCreature => 1,
                },
                matches!(spec.form, PermanentForm::AddCreature),
            )
        })
        .collect();
    if let Some((timestamp, _)) = source_counter_turn_creature(world, id) {
        changes.push((timestamp, 1, true));
    }
    changes.extend(land_forms(world, id).map(|row| (row.timestamp, 2, false)));
    for row in animations(world, id) {
        if row.food_only {
            changes.push((row.timestamp, 16, false));
            continue;
        }
        if row.artifact_noncreature || row.enchantment_noncreature {
            let kind = if row.enchantment_noncreature { 32 } else { 16 };
            changes.push((row.timestamp, kind | (result & (4 | 8)), false));
            continue;
        }
        let added = if row.preserve_card_types {
            if row.artifact {
                16
            } else {
                continue;
            }
        } else if row.artifact {
            17
        } else {
            1
        };
        changes.push((row.timestamp, added | if row.land { 2 } else { 0 }, true));
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
    for (_, mask, additive) in changes {
        if additive {
            result |= mask;
        } else {
            result = mask;
        }
    }
    result
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn name<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> &'static str {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || world
            .roles()
            .is_none_or(|r| r.characteristic_sources.is_empty())
    {
        return if object.zone == Zone::Stack {
            world.game().effective_spell_card(id).name()
        } else {
            world.copiable_name(id)
        };
    }
    changed_name(world, id)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_name<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> &'static str {
    aura_effects(world, id)
        .into_iter()
        .filter_map(|(t, s)| s.name.map(|name| (t, name)))
        .max_by_key(|e| e.0)
        .map_or_else(|| world.copiable_name(id), |e| e.1)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[inline]
pub(super) fn colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    let object = world.object(id);
    if object.zone != Zone::Battlefield
        || world.roles().is_none_or(|r| {
            r.characteristic_sources.is_empty() && r.animations.iter().all(|a| a.colors.is_none())
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn changed_colors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> u8 {
    aura_effects(world, id)
        .into_iter()
        .filter_map(|(timestamp, spec)| {
            if matches!(spec.form, PermanentForm::AddCreature) {
                return None;
            }
            Some((
                timestamp,
                spec.colors
                    .iter()
                    .fold(0, |mask, c| mask | (1 << c.index())),
            ))
        })
        .chain(
            animations(world, id)
                .filter_map(|row| row.colors.map(|colors| (row.timestamp, colors))),
        )
        .max_by_key(|row| row.0)
        .map_or_else(|| world.object(id).card.colors(), |(_, colors)| colors)
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
    All,
    LandOnly,
    BasicLand(crate::cards::BasicLandType),
    AddBasicLand(crate::cards::BasicLandType),
    FoodOnly,
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
        && source_counter_turn_creature(world, id).is_none()
        && land_forms(world, id).next().is_none()
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
    let mut changes: Vec<_> = aura_effects(world, id)
        .into_iter()
        .map(|(time, s)| {
            (
                time,
                match s.form {
                    PermanentForm::Creature => SubtypeChange::Named(s.creature_types, true),
                    PermanentForm::Land => SubtypeChange::LandOnly,
                    PermanentForm::AddCreature => SubtypeChange::Add(s.creature_types),
                },
            )
        })
        .collect();
    if let Some((timestamp, types)) = source_counter_turn_creature(world, id)
        && !types.is_empty()
    {
        changes.push((timestamp, SubtypeChange::Named(types, false)));
    }
    changes.extend(land_forms(world, id).map(|row| (row.timestamp, SubtypeChange::LandOnly)));
    changes.extend(animations(world, id).filter_map(|row| {
        if row.food_only {
            return Some((row.timestamp, SubtypeChange::FoodOnly));
        }
        // Artifact-only form filters correlated subtypes against final layer-four
        // card types in the shared subtype projection; later animation may restore them.
        if row.artifact_noncreature || row.enchantment_noncreature {
            return None;
        }
        // A subtype-only form cannot give creature types to a noncreature/nonkindred.
        if row.preserve_card_types && world.kinds(id) & (1 | 128) == 0 {
            return None;
        }
        if row.all_creature_types {
            Some((row.timestamp, SubtypeChange::All))
        } else {
            row.creature_types
                .map(|t| (row.timestamp, SubtypeChange::Named(t, false)))
        }
    }));
    changes
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
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
    pub(super) fn animate_source_colored(
        &mut self,
        item: &SpellView,
        spec: AnimationSpec,
        colors: u8,
    ) {
        if let Some(source) = item.source {
            self.change_object_form_colored(
                source.object,
                spec,
                AnimationTypeMode::AddCreature,
                Some(colors),
            );
        }
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
        for target in &item.targets {
            if self.same_target_object(*target)
                && let Some(object) = target.object
            {
                self.change_object_form(object, spec, AnimationTypeMode::PreserveCardTypes);
            }
        }
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
    pub(super) fn animate_targets(&mut self, item: &SpellView, spec: AnimationSpec) {
        for target in &item.targets {
            if self.same_target_object(*target)
                && let Some(object) = target.object
            {
                self.change_object_form(object, spec, AnimationTypeMode::AddCreature);
            }
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
        self.change_object_form_colored(object, spec, type_mode, None);
    }
    fn change_object_form_colored(
        &mut self,
        object: ObjectRef,
        spec: AnimationSpec,
        type_mode: AnimationTypeMode,
        colors: Option<u8>,
    ) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let row = AnimationView {
            object,
            timestamp,
            artifact: spec.artifact,
            land: matches!(type_mode, AnimationTypeMode::AddCreatureAndLand),
            preserve_card_types: matches!(type_mode, AnimationTypeMode::PreserveCardTypes),
            food_only: false,
            artifact_noncreature: false,
            enchantment_noncreature: false,
            colors,
            creature_types: match spec.creature_types {
                AnimationCreatureTypes::Named(t) => Some(t),
                _ => None,
            },
            all_creature_types: matches!(spec.creature_types, AnimationCreatureTypes::All),
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
    pub(super) fn set_creature_form(
        &mut self,
        item: &SpellView,
        types: &'static [&'static str],
        colors: u8,
        base: AmountValue,
    ) {
        let value =
            i128::try_from(self.item_amount(base, item)).capacity("creature form base exhausted");
        for id in self.keyword_subjects(item, Subject::Targets) {
            let row = AnimationView {
                object: self.object_ref(id),
                timestamp: self.next_role_timestamp(),
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                enchantment_noncreature: false,
                colors: Some(colors),
                creature_types: Some(types),
                all_creature_types: false,
                base: Some((value, value)),
                keywords: Keywords::NONE,
                expires_turn: Some(self.state.turn),
            };
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .animations
                .push(row);
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

/// Same immutable form record for actual and speculative entry worlds.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn food_form(object: ObjectRef, timestamp: u64) -> AnimationView {
    AnimationView {
        object,
        timestamp,
        artifact: true,
        land: false,
        preserve_card_types: false,
        food_only: true,
        artifact_noncreature: false,
        enchantment_noncreature: false,
        colors: None,
        creature_types: None,
        all_creature_types: false,
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
