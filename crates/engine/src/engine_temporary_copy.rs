//! Cold, exact-incarnation layer-one copies. A blueprint is frozen and flat:
//! later changes to the copied source never change an existing layer.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CopyTypes {
    pub replace: bool,
    pub human_villain: bool,
    pub mercenary: bool,
    pub nightmare: bool,
    pub golem: bool,
    pub fractal: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub zombie: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub mutant: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyOverrides {
    pub name: Option<&'static str>,
    pub legendary: Option<bool>,
    pub kinds: Option<u16>,
    pub types: Option<CopyTypes>,
    pub keywords: Keywords,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CopyOverrides {
    pub(super) const NONE: Self = Self {
        name: None,
        legendary: None,
        kinds: None,
        types: None,
        keywords: Keywords::NONE,
    };
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CopyDeadline {
    EndOfTurn,
    NextEndStep,
    NextTurn { player: Player, created_turn: u32 },
    WhileSource { source: ObjectRef },
    WhileAttached { source: ObjectRef, origin: u64 },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CopyDeadline {
    pub(super) fn source(self) -> Option<ObjectRef> {
        match self {
            Self::WhileSource { source } | Self::WhileAttached { source, .. } => Some(source),
            _ => None,
        }
    }
    fn expired(self, turn: u32, active: Player, cleanup: bool) -> bool {
        match self {
            Self::EndOfTurn => cleanup,
            Self::NextEndStep | Self::WhileSource { .. } | Self::WhileAttached { .. } => false,
            Self::NextTurn {
                player,
                created_turn,
            } => !cleanup && turn > created_turn && active == player,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RetainedActivationOrigin {
    pub ability_origin: Option<u64>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyLayerView {
    pub recipe: CopyRecipe,
    pub origin: u64,
    /// Static-effect chronology can change without acquiring a new ability.
    pub provider_timestamp: u64,
    pub deadline: CopyDeadline,
    /// The independent timestamp of this layer's hand-setting ability, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_link_origin: Option<RetainedActivationOrigin>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TemporaryCopyView {
    pub physical_card: Card,
    pub physical_token: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_hand_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_provider_timestamp: Option<u64>,
    pub active_origin: u64,
    pub layers: Vec<CopyLayerView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Layer {
    pub value: CopyLayerView,
    pub used_abilities: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub object: ObjectRef,
    pub physical_card: Card,
    pub physical_kind: CopyKind,
    pub original_recipe: Option<CopyRecipe>,
    pub original_used: u64,
    pub original_provider_time: Option<u64>,
    pub original_hand_time: Option<u64>,
    pub layers: Vec<Layer>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Row {
    pub(super) fn as_entry(
        object: ObjectRef,
        physical_card: Card,
        physical_kind: CopyKind,
        original_recipe: Option<CopyRecipe>,
        recipe: CopyRecipe,
        origin: u64,
    ) -> Self {
        Self {
            object,
            physical_card,
            physical_kind,
            original_recipe,
            original_used: 0,
            original_provider_time: None,
            original_hand_time: None,
            layers: vec![Layer {
                value: CopyLayerView {
                    recipe,
                    origin,
                    provider_timestamp: 0,
                    deadline: CopyDeadline::WhileSource { source: object },
                    hand_timestamp: None,
                    retained_link_origin: None,
                },
                used_abilities: 0,
            }],
        }
    }
    pub(super) fn active(&self) -> &Layer {
        self.layers.last().expect("live temporary copy has a layer")
    }
    pub(super) fn view(&self) -> TemporaryCopyView {
        TemporaryCopyView {
            physical_card: self.physical_card.front_face(),
            physical_token: self.physical_kind == CopyKind::Token || self.physical_card.is_token(),
            original_hand_timestamp: self.original_hand_time,
            original_provider_timestamp: self.original_provider_time,
            active_origin: self.active().value.origin,
            layers: self.layers.iter().map(|layer| layer.value).collect(),
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<Row>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<Row>>()
            + rows.len() * std::mem::size_of::<Row>()
            + rows
                .iter()
                .map(|row| row.layers.len() * std::mem::size_of::<Layer>())
                .sum::<usize>()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn copy_targets_from_cast_spell(&mut self, item: &SpellView) {
        let Some(&recipe) = item.captured_artifact_copy.as_deref() else {
            return;
        };
        for &bound in &item.targets {
            if let Some(recipient) = bound.object
                && self.same_target_object(bound)
                && self.on_battlefield(recipient.card)
            {
                self.apply_temporary_copy(recipient, recipe, CopyDeadline::EndOfTurn);
            }
        }
    }

    pub(super) fn remembered_exiled_creature_copy(
        &self,
        item: &SpellView,
    ) -> Option<(ObjectRef, CopyRecipe)> {
        let source = item.source?.object;
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return None;
        }
        self.state
            .resolving
            .as_ref()?
            .remembered_objects
            .as_ref()?
            .iter()
            .find_map(|&blueprint| {
                let object = &self.state.objects[blueprint.card as usize];
                (self.object_ref(blueprint.card) == blueprint
                    && object.zone == Zone::Exile
                    && object.is_card()
                    && object.card.is_creature())
                .then(|| (source, self.copiable_recipe(blueprint)))
            })
    }

    pub(super) fn resolve_in_place_copy(
        &mut self,
        item: &SpellView,
        spec: crate::cards::InPlaceCopySpec,
    ) {
        let selected = |group| {
            item.targets.iter().find_map(|bound| {
                (bound.group == group && self.same_target_object(*bound))
                    .then_some(bound.object)
                    .flatten()
            })
        };
        let Some(recipient) = spec
            .recipient_group
            .map_or(item.source.map(|source| source.object), selected)
        else {
            return;
        };
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        let Some(blueprint) = selected(spec.blueprint_group) else {
            return;
        };
        let object = &self.state.objects[blueprint.card as usize];
        let mut recipe = match object.zone {
            Zone::Battlefield => self.copiable_recipe(blueprint),
            Zone::Graveyard if object.is_card() => CopyRecipe::new(object.card),
            _ => return,
        };
        let old = recipe.overrides.unwrap_or(CopyOverrides::NONE);
        let incoming = spec.overrides;
        if incoming.types.is_some_and(|types| types.replace) {
            recipe.replacement_subtype = None;
            recipe.exceptions.spider_human_hero = false;
            recipe.exceptions.shapeshifter_rogue = false;
            recipe.exceptions.bird = false;
            recipe.exceptions.reflection = false;
        }
        let types = incoming
            .types
            .map(|value| {
                if value.replace {
                    return value;
                }
                let previous = recipe.copy_types();
                CopyTypes {
                    replace: previous.replace,
                    human_villain: previous.human_villain || value.human_villain,
                    mercenary: previous.mercenary || value.mercenary,
                    nightmare: previous.nightmare || value.nightmare,
                    golem: previous.golem || value.golem,
                    fractal: previous.fractal || value.fractal,
                    zombie: previous.zombie || value.zombie,
                    mutant: previous.mutant || value.mutant,
                }
            })
            .or(old.types);
        recipe.overrides = Some(CopyOverrides {
            name: incoming.name.or(old.name),
            legendary: incoming.legendary.or(old.legendary),
            kinds: if spec.add_kinds != 0 {
                Some(incoming.kinds.unwrap_or_else(|| recipe.kinds()) | spec.add_kinds)
            } else {
                incoming.kinds.or(old.kinds)
            },
            types,
            keywords: old.keywords.union(incoming.keywords),
        });
        if let Some(base) = spec.base {
            recipe = recipe.with_exceptions(CopyExceptions {
                base: Some(base),
                ..CopyExceptions::NONE
            });
        }
        let deadline = if spec.next_turn {
            CopyDeadline::NextTurn {
                player: item.controller,
                created_turn: self.state.turn,
            }
        } else {
            CopyDeadline::EndOfTurn
        };
        self.apply_temporary_copy(recipient, recipe, deadline);
    }
    pub(super) fn copy_target_to_other_controlled_creatures(&mut self, item: &SpellView) {
        self.copy_target_to_controlled_cohort(item, false);
    }
    pub(super) fn copy_target_to_controlled_cohort(&mut self, item: &SpellView, nonlands: bool) {
        let Some(blueprint) = item.targets.iter().find_map(|bound| {
            bound.object.filter(|reference| {
                self.same_target_object(*bound) && self.on_battlefield(reference.card)
            })
        }) else {
            return;
        };
        let recipe = self.copiable_recipe(blueprint);
        // Both the recipe and recipients precede any copy/provider changes.
        let recipients = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && object.controller == item.controller
                    && if nonlands {
                        self.current_kinds(id) & 2 == 0
                    } else {
                        self.is_creature(id) && self.object_ref(id) != blueprint
                    })
                .then_some(self.object_ref(id))
            })
            .collect::<Vec<_>>();
        self.apply_temporary_copy_cohort(recipients, recipe, |recipient| {
            if nonlands {
                CopyDeadline::WhileSource { source: recipient }
            } else {
                CopyDeadline::EndOfTurn
            }
        });
    }
    pub(super) fn copy_to_controlled_subtype_until_end_step(
        &mut self,
        controller: Player,
        subtype: &'static str,
        recipe: CopyRecipe,
    ) {
        let recipients = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && object.controller == controller
                    && self.has_subtype(id, subtype))
                .then_some(self.object_ref(id))
            })
            .collect();
        self.apply_temporary_copy_cohort(recipients, recipe, |_| CopyDeadline::NextEndStep);
    }
    fn apply_temporary_copy_cohort(
        &mut self,
        recipients: Vec<ObjectRef>,
        recipe: CopyRecipe,
        deadline: impl Fn(ObjectRef) -> CopyDeadline,
    ) {
        if recipients.is_empty() {
            return;
        }
        let control_watch = self.freeze_control_watch();
        let provider_time = self.next_role_timestamp();
        let hand_time = self.new_copy_hand_timestamp(recipe.definition);
        for &recipient in &recipients {
            self.apply_temporary_copy_timestamped(
                recipient,
                recipe,
                deadline(recipient),
                Some((provider_time, hand_time)),
                false,
            );
        }
        // No intermediate recipient prefix is a game world. In particular,
        // control changes, enduring designations and ability acquisitions can
        // have irreversible consequences, so project only the complete cohort.
        self.refresh_controllers_after(Some(control_watch));
        for recipient in recipients {
            if !self.is_face_down(recipient) {
                self.renew_copy_grant_origin(recipient, true);
            }
        }
        // The final controller/story projection may enable other providers too.
        self.refresh_copy_grant_availability();
        self.refresh_combat_battle_types();
    }
    pub(super) fn temporary_copy_row(&self, object: ObjectRef) -> Option<&Row> {
        self.state
            .temporary_copies
            .as_ref()?
            .iter()
            .find(|row| row.object == object)
    }
    pub(super) fn copy_ability_origin(&self, object: ObjectRef) -> Option<u64> {
        if self.is_face_down(object) {
            return None;
        }
        self.temporary_copy_row(object)?
            .layers
            .last()
            .map(|layer| layer.value.origin)
    }
    pub(super) fn retains_ability_origin(&self, object: ObjectRef, origin: u64) -> bool {
        self.temporary_copy_row(object)
            .is_some_and(|row| row.layers.iter().any(|layer| layer.value.origin == origin))
            || self.state.temporary_abilities.as_ref().is_some_and(|rows| {
                rows.iter()
                    .any(|grant| grant.recipient == object && grant.origin == Some(origin))
            })
            || self.retains_provided_origin(origin)
    }
    pub(super) fn physical_definition(&self, id: CardId) -> Card {
        self.temporary_copy_row(self.object_ref(id))
            .map_or(self.state.objects[id as usize].card, |row| {
                row.physical_card
            })
    }
    pub(super) fn restore_projected_copy_identity(
        &self,
        reference: ObjectRef,
        object: &mut Object,
    ) {
        if let Some(row) = self.temporary_copy_row(reference) {
            object.card = row.physical_card;
            object.copy_kind = row.physical_kind;
        }
    }
    #[cfg(feature = "test-support")]
    pub(super) fn temporary_copy_view(&self, object: ObjectRef) -> Option<TemporaryCopyView> {
        self.temporary_copy_row(object).map(Row::view)
    }
    fn copy_provider_time(&self, object: ObjectRef) -> Option<u64> {
        let roles = self.state.roles.as_ref()?;
        roles
            .ability_timestamps
            .iter()
            .find(|row| row.source == object)
            .map(|row| row.timestamp)
            .or_else(|| {
                roles
                    .base_stats
                    .iter()
                    .find(|row| row.static_source && row.source == object)
                    .map(|row| row.timestamp)
            })
            .or_else(|| {
                roles
                    .live_static
                    .iter()
                    .find(|row| row.source == object)
                    .map(|row| row.timestamp)
            })
    }
    fn copy_hand_time(&self, object: ObjectRef) -> Option<u64> {
        self.state
            .hand_size
            .as_ref()?
            .sources
            .iter()
            .find(|row| row.source == object)
            .map(|row| row.timestamp)
    }
    fn refresh_copy_providers(
        &mut self,
        object: ObjectRef,
        time: Option<u64>,
        hand_time: Option<u64>,
    ) {
        self.forget_hand_size_source(object);
        self.remove_face_providers(object.card);
        self.register_battlefield_providers(object.card);
        if let (Some(time), Some(roles)) = (time, &mut self.state.roles) {
            for row in &mut roles.ability_timestamps {
                if row.source == object {
                    row.timestamp = time;
                }
            }
            for row in &mut roles.base_stats {
                if row.static_source && row.source == object {
                    row.timestamp = time;
                }
            }
            for row in &mut roles.live_static {
                if row.source == object {
                    row.timestamp = time;
                }
            }
        }
        if let (Some(time), Some(state)) = (hand_time, &mut self.state.hand_size) {
            for row in &mut state.sources {
                if row.source == object {
                    row.timestamp = time;
                }
            }
        }
    }
    fn materialize_copy_row(&mut self, row: &Row, refresh: bool) {
        // Layer 1b masks all copy effects while the permanent remains face down.
        if self.is_face_down(row.object) {
            return;
        }
        let active = row.layers.last();
        let recipe = active
            .map(|layer| {
                let physical = row
                    .original_recipe
                    .unwrap_or_else(|| CopyRecipe::new(row.physical_card));
                layer
                    .value
                    .recipe
                    .with_copy_effect_faces(physical.face_structure())
            })
            .or(row.original_recipe);
        let definition = active.map_or(row.physical_card, |layer| layer.value.recipe.definition);
        let kind = if active.is_some() {
            if row.physical_kind == CopyKind::Token || row.physical_card.is_token() {
                CopyKind::Token
            } else {
                CopyKind::Physical
            }
        } else {
            row.physical_kind
        };
        self.clear_copy_recipe(row.object);
        if let Some(recipe) = recipe {
            self.set_copy_recipe(row.object, recipe);
        }
        let object = &mut self.state.objects[row.object.card as usize];
        object.card = definition;
        object.copy_kind = kind;
        object.used_abilities = active.map_or(row.original_used, |layer| layer.used_abilities);
        if refresh {
            self.refresh_copy_providers(
                row.object,
                active
                    .map(|layer| layer.value.provider_timestamp)
                    .or(row.original_provider_time),
                active.map_or(row.original_hand_time, |layer| layer.value.hand_timestamp),
            );
        }
    }
    /// Install the already-chosen layer before entry providers/events inspect the entrant.
    pub(super) fn install_entry_copy(
        &mut self,
        object: ObjectRef,
        physical_card: Card,
        recipe: CopyRecipe,
    ) {
        let origin = self.state.roles.get_or_insert_default().next_acquisition();
        let row = Row::as_entry(
            object,
            physical_card,
            self.state.objects[object.card as usize].copy_kind,
            self.copy_recipe(object),
            recipe,
            origin,
        );
        self.state
            .temporary_copies
            .get_or_insert_default()
            .push(row.clone());
        self.materialize_copy_row(&row, false);
    }
    pub(super) fn apply_temporary_copy(
        &mut self,
        object: ObjectRef,
        recipe: CopyRecipe,
        deadline: CopyDeadline,
    ) {
        self.apply_temporary_copy_timestamped(object, recipe, deadline, None, true);
    }
    fn apply_temporary_copy_timestamped(
        &mut self,
        object: ObjectRef,
        recipe: CopyRecipe,
        deadline: CopyDeadline,
        shared_time: Option<(u64, Option<u64>)>,
        project_world: bool,
    ) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        if let Some(source) = deadline.source()
            && (!self.on_battlefield(source.card) || self.object_ref(source.card) != source)
        {
            return;
        }
        let control_watch = project_world.then(|| self.freeze_control_watch());
        let origin = self.next_role_timestamp();
        let old_used = self.state.objects[object.card as usize].used_abilities;
        let initial = if self.temporary_copy_row(object).is_none() {
            let physical = &self.state.objects[object.card as usize];
            Some(Row {
                object,
                physical_card: physical.card,
                physical_kind: physical.copy_kind,
                original_recipe: self.copy_recipe(object),
                original_used: old_used,
                original_provider_time: self.copy_provider_time(object),
                original_hand_time: self.copy_hand_time(object),
                layers: Vec::new(),
            })
        } else {
            None
        };
        let rows = self.state.temporary_copies.get_or_insert_default();
        if let Some(row) = initial {
            rows.push(row);
        }
        let row = rows.iter_mut().find(|row| row.object == object).unwrap();
        if let Some(active) = row.layers.last_mut() {
            active.used_abilities = old_used;
        }
        row.layers.push(Layer {
            value: CopyLayerView {
                recipe,
                origin,
                // One effect can acquire abilities independently on several
                // objects while giving all their static effects one timestamp.
                provider_timestamp: shared_time.map_or(origin, |time| time.0),
                deadline,
                hand_timestamp: shared_time.and_then(|time| time.1),
                retained_link_origin: None,
            },
            used_abilities: 0,
        });
        let row = row.clone();
        self.materialize_copy_row(&row, true);
        // Registration allocates a timestamp only for a hand-setting ability.
        // Preserve it with this acquisition while the layer is hidden.
        let hand_time = self.copy_hand_time(object);
        self.state
            .temporary_copies
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row.object == object)
            .unwrap()
            .layers
            .last_mut()
            .unwrap()
            .value
            .hand_timestamp = hand_time;
        if project_world {
            if !self.is_face_down(object) {
                self.renew_copy_grant_origin(object, true);
            }
            self.refresh_controllers_after(control_watch);
            self.refresh_combat_battle_types();
        }
    }
    pub(super) fn reveal_temporary_copy(&mut self, object: ObjectRef, physical: Card) -> bool {
        let Some(row) = self
            .state
            .temporary_copies
            .as_mut()
            .and_then(|rows| rows.iter_mut().find(|row| row.object == object))
        else {
            return false;
        };
        row.physical_card = physical;
        let row = row.clone();
        self.materialize_copy_row(&row, true);
        self.renew_copy_grant_origin(object, true);
        true
    }
    /// CR613.7a/e/g: a real new object timestamp updates its static effects in
    /// their prior relative order, without replacing any ability instance.
    pub(super) fn retimestamp_temporary_copy(&mut self, object: ObjectRef) -> bool {
        let Some(mut row) = self.temporary_copy_row(object).cloned() else {
            return false;
        };
        row.layers.last_mut().unwrap().used_abilities =
            self.state.objects[object.card as usize].used_abilities;
        row.original_provider_time = Some(self.next_role_timestamp());
        row.original_hand_time = self.new_copy_hand_timestamp(row.physical_card);
        for layer in &mut row.layers {
            layer.value.provider_timestamp = self.next_role_timestamp();
            layer.value.hand_timestamp =
                self.new_copy_hand_timestamp(layer.value.recipe.definition);
        }
        *self
            .state
            .temporary_copies
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|saved| saved.object == object)
            .unwrap() = row.clone();
        self.materialize_copy_row(&row, true);
        self.retimestamp_self_granted_statics(object);
        true
    }
    pub(super) fn transform_under_temporary_copy(
        &mut self,
        object: ObjectRef,
        other: Card,
    ) -> bool {
        let Some(rows) = &mut self.state.temporary_copies else {
            return false;
        };
        let Some(row) = rows.iter_mut().find(|row| row.object == object) else {
            return false;
        };
        row.physical_card = other;
        if let Some(recipe) = &mut row.original_recipe {
            assert_eq!(recipe.turn_face(), Some(other));
        }
        self.retimestamp_temporary_copy(object);
        true
    }
    pub(super) fn forget_temporary_copy(&mut self, object: ObjectRef) {
        let Some(rows) = &self.state.temporary_copies else {
            return;
        };
        let Some(index) = rows.iter().position(|row| row.object == object) else {
            return;
        };
        let mut row = self.state.temporary_copies.as_mut().unwrap().remove(index);
        row.layers.clear();
        self.materialize_copy_row(&row, false);
        if self.state.temporary_copies.as_ref().unwrap().is_empty() {
            self.state.temporary_copies = None;
        }
    }
    /// Called only after the complete movement batch committed. Departure LKI
    /// was frozen before any recipient or lifetime-source moved.
    pub(super) fn prune_source_bound_copies(&mut self) -> bool {
        let roles = self.state.roles.as_deref();
        let Some(rows) = &mut self.state.temporary_copies else {
            return false;
        };
        let objects = &self.state.objects;
        let mut changes = Vec::new();
        for row in rows.iter_mut() {
            if !row
                .layers
                .iter()
                .any(|layer| layer.value.deadline.source().is_some())
            {
                continue;
            }
            let old = row.active().value.origin;
            let old_len = row.layers.len();
            row.layers.last_mut().unwrap().used_abilities =
                objects[row.object.card as usize].used_abilities;
            row.layers.retain(|layer| {
                let deadline = layer.value.deadline;
                deadline.source().is_none_or(|source| {
                    let source_object = &objects[source.card as usize];
                    source_object.zone == Zone::Battlefield
                        && source_object.generation == source.generation
                        && match deadline {
                            CopyDeadline::WhileAttached { origin, .. } => {
                                source_object.attached_to == Some(row.object)
                                    && roles.and_then(|roles| roles.attachment_origin(source))
                                        == Some(origin)
                            }
                            _ => true,
                        }
                })
            });
            if row.layers.len() != old_len
                && row.layers.last().map(|layer| layer.value.origin) != Some(old)
            {
                changes.push(row.clone());
            }
        }
        self.restore_copy_layers(&changes);
        if !changes.is_empty() {
            self.refresh_controllers();
        }
        !changes.is_empty()
    }
    pub(super) fn expire_temporary_copies(&mut self, cleanup: bool) -> bool {
        let turn = self.state.turn;
        let active = self.state.active;
        self.expire_copy_layers(|deadline| deadline.expired(turn, active, cleanup), !cleanup)
    }
    pub(super) fn expire_next_end_step_copies(&mut self) {
        self.expire_copy_layers(|deadline| deadline == CopyDeadline::NextEndStep, true);
    }
    fn expire_copy_layers(
        &mut self,
        expires: impl Fn(CopyDeadline) -> bool,
        refresh_control: bool,
    ) -> bool {
        if self.state.temporary_copies.is_none() {
            return false;
        }
        let control_watch = refresh_control.then(|| self.freeze_control_watch());
        let rows = self.state.temporary_copies.as_mut().unwrap();
        let mut changes = Vec::new();
        for row in rows.iter_mut() {
            let old_origin = row.active().value.origin;
            row.layers.last_mut().unwrap().used_abilities =
                self.state.objects[row.object.card as usize].used_abilities;
            row.layers.retain(|layer| !expires(layer.value.deadline));
            if row.layers.last().map(|layer| layer.value.origin) != Some(old_origin) {
                changes.push(row.clone());
            }
        }
        self.restore_copy_layers(&changes);
        if refresh_control && !changes.is_empty() {
            self.refresh_controllers_after(control_watch);
        }
        !changes.is_empty()
    }
    fn restore_copy_layers(&mut self, changes: &[Row]) {
        // Restore the whole cohort before refreshing any abilities or providers.
        for row in changes {
            self.materialize_copy_row(row, false);
        }
        if let Some(rows) = &mut self.state.temporary_copies {
            rows.retain(|row| !row.layers.is_empty());
            if rows.is_empty() {
                self.state.temporary_copies = None;
            }
        }
        for row in changes {
            self.materialize_copy_row(row, true);
        }
        for row in changes {
            if !self.is_face_down(row.object) {
                self.renew_copy_grant_origin(row.object, false);
            }
        }
    }
    pub(super) fn reset_copy_turn_uses(&mut self) {
        if let Some(rows) = &mut self.state.temporary_copies {
            for row in rows.iter_mut() {
                row.original_used = row
                    .physical_card
                    .retain_lifetime_activation_uses(row.original_used);
                for layer in &mut row.layers {
                    layer.used_abilities = layer
                        .value
                        .recipe
                        .definition
                        .retain_lifetime_activation_uses(layer.used_abilities);
                }
            }
        }
    }
    pub(super) fn record_copy_activation_use(&mut self, item: &SpellView, ability: u16) {
        let Some(source) = item.source else {
            return;
        };
        let bit = 1u64.checked_shl(u32::from(ability)).unwrap_or(0);
        if self.copy_ability_origin(source.object) == item.source_ability_origin {
            self.state.objects[source.id as usize].used_abilities |= bit;
        } else if let Some(rows) = &mut self.state.temporary_copies
            && let Some(row) = rows.iter_mut().find(|row| row.object == source.object)
        {
            if let Some(origin) = item.source_ability_origin {
                if let Some(layer) = row
                    .layers
                    .iter_mut()
                    .find(|layer| layer.value.origin == origin)
                {
                    layer.used_abilities |= bit;
                }
            } else {
                row.original_used |= bit;
            }
        }
    }
}
