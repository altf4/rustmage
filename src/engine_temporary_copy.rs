//! Cold, exact-incarnation layer-one copies. A blueprint is frozen and flat:
//! later changes to the copied source never change an existing layer.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CopyTypes {
    pub replace: bool,
    pub human_villain: bool,
    pub mercenary: bool,
    pub nightmare: bool,
    pub golem: bool,
    pub fractal: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyOverrides {
    pub name: Option<&'static str>,
    pub legendary: Option<bool>,
    pub kinds: Option<u16>,
    pub types: Option<CopyTypes>,
    pub keywords: Keywords,
}
impl CopyOverrides {
    pub(super) const NONE: Self = Self {
        name: None,
        legendary: None,
        kinds: None,
        types: None,
        keywords: Keywords::NONE,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CopyDeadline {
    EndOfTurn,
    NextTurn { player: Player, created_turn: u32 },
    WhileSource { source: ObjectRef },
}
impl CopyDeadline {
    pub(super) fn source(self) -> Option<ObjectRef> {
        match self {
            Self::WhileSource { source } => Some(source),
            _ => None,
        }
    }
    fn expired(self, turn: u32, active: Player, cleanup: bool) -> bool {
        match self {
            Self::EndOfTurn => cleanup,
            Self::WhileSource { .. } => false,
            Self::NextTurn {
                player,
                created_turn,
            } => !cleanup && turn > created_turn && active == player,
        }
    }
}
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
}
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Layer {
    pub value: CopyLayerView,
    pub used_abilities: u64,
}
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
impl Row {
    pub(super) fn active(&self) -> &Layer {
        self.layers.last().expect("live temporary copy has a layer")
    }
    pub(super) fn expire(&mut self, turn: u32, active: Player, cleanup: bool) -> bool {
        let old = self.layers.len();
        self.layers
            .retain(|layer| !layer.value.deadline.expired(turn, active, cleanup));
        old != self.layers.len()
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
pub(super) type Rows = Option<SharedVec<Row>>;
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

impl Game {
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
        let Some(recipient) = spec.recipient_group.map_or(Some(item.object), selected) else {
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
    pub(super) fn apply_temporary_copy(
        &mut self,
        object: ObjectRef,
        recipe: CopyRecipe,
        deadline: CopyDeadline,
    ) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        if let Some(source) = deadline.source()
            && (!self.on_battlefield(source.card) || self.object_ref(source.card) != source)
        {
            return;
        }
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
                provider_timestamp: origin,
                deadline,
                hand_timestamp: None,
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
        if !self.is_face_down(object) {
            self.renew_copy_grant_origin(object, true);
        }
        self.refresh_controllers();
        self.refresh_combat_battle_types();
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
        self.refresh_controllers();
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
                layer.value.deadline.source().is_none_or(|source| {
                    objects[source.card as usize].zone == Zone::Battlefield
                        && objects[source.card as usize].generation == source.generation
                })
            });
            if row.layers.len() != old_len
                && row.layers.last().map(|layer| layer.value.origin) != Some(old)
            {
                changes.push(row.clone());
            }
        }
        // Restore every definition first; no provider may inspect an empty old
        // layer or a half-restored simultaneous set of other recipients.
        for row in &changes {
            self.materialize_copy_row(row, false);
        }
        if let Some(rows) = &mut self.state.temporary_copies {
            rows.retain(|row| !row.layers.is_empty());
            if rows.is_empty() {
                self.state.temporary_copies = None;
            }
        }
        for row in &changes {
            self.materialize_copy_row(row, true);
            if !self.is_face_down(row.object) {
                self.renew_copy_grant_origin(row.object, false);
            }
        }
        if !changes.is_empty() {
            self.refresh_controllers();
        }
        !changes.is_empty()
    }
    pub(super) fn expire_temporary_copies(&mut self, cleanup: bool) -> bool {
        let Some(rows) = &mut self.state.temporary_copies else {
            return false;
        };
        let mut changes = Vec::new();
        for row in rows.iter_mut() {
            let old_origin = row.active().value.origin;
            row.layers.last_mut().unwrap().used_abilities =
                self.state.objects[row.object.card as usize].used_abilities;
            if row.expire(self.state.turn, self.state.active, cleanup)
                && row.layers.last().map(|layer| layer.value.origin) != Some(old_origin)
            {
                changes.push(row.clone());
            }
        }
        for row in &changes {
            self.materialize_copy_row(row, true);
            if !self.is_face_down(row.object) {
                self.renew_copy_grant_origin(row.object, false);
            }
        }
        if let Some(rows) = &mut self.state.temporary_copies {
            rows.retain(|row| !row.layers.is_empty());
            if rows.is_empty() {
                self.state.temporary_copies = None;
            }
        }
        if !changes.is_empty() {
            self.refresh_controllers();
        }
        !changes.is_empty()
    }
    pub(super) fn reset_copy_turn_uses(&mut self) {
        if let Some(rows) = &mut self.state.temporary_copies {
            for row in rows.iter_mut() {
                row.original_used &= row.physical_card.exhaust_mask();
                for layer in &mut row.layers {
                    layer.used_abilities &= layer.value.recipe.definition.exhaust_mask();
                }
            }
        }
    }
    pub(super) fn record_copy_activation_use(&mut self, item: &SpellView, ability: u16) {
        let bit = 1u64 << ability;
        if self.copy_ability_origin(item.object) == item.source_ability_origin {
            self.state.objects[item.card as usize].used_abilities |= bit;
        } else if let Some(rows) = &mut self.state.temporary_copies
            && let Some(row) = rows.iter_mut().find(|row| row.object == item.object)
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
