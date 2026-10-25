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
}
impl CopyDeadline {
    fn expired(self, turn: u32, active: Player, cleanup: bool) -> bool {
        match self {
            Self::EndOfTurn => cleanup,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn msh_temporary_copy_deadlines_use_captured_controller_and_actual_next_turn() {
        for player in [Player::P0, Player::P1] {
            let deadline = CopyDeadline::NextTurn {
                player,
                created_turn: 6,
            };
            assert!(!deadline.expired(6, player, false));
            assert!(!deadline.expired(7, player.other(), false));
            assert!(!deadline.expired(8, player, true));
            assert!(deadline.expired(8, player, false));
        }
        assert!(CopyDeadline::EndOfTurn.expired(1, Player::P0, true));
        assert!(!CopyDeadline::EndOfTurn.expired(1, Player::P0, false));
    }
    #[test]
    fn msh_temporary_copy_owned_layers_reveal_older_origin_without_rebinding_source() {
        let card = Card::from_name("Hill Giant").unwrap();
        let mut row = Row {
            object: ObjectRef {
                card: 17,
                generation: 9,
            },
            physical_card: card,
            physical_kind: CopyKind::None,
            original_recipe: None,
            original_used: 5,
            original_provider_time: None,
            original_hand_time: None,
            layers: vec![
                Layer {
                    value: CopyLayerView {
                        recipe: CopyRecipe::new(card),
                        origin: 11,
                        provider_timestamp: 11,
                        deadline: CopyDeadline::NextTurn {
                            player: Player::P1,
                            created_turn: 2,
                        },
                        hand_timestamp: None,
                    },
                    used_abilities: 3,
                },
                Layer {
                    value: CopyLayerView {
                        recipe: CopyRecipe::new(card),
                        origin: 12,
                        provider_timestamp: 12,
                        deadline: CopyDeadline::EndOfTurn,
                        hand_timestamp: None,
                    },
                    used_abilities: 0,
                },
            ],
        };
        let branch = row.clone();
        assert!(row.expire(2, Player::P0, true));
        assert_eq!(row.active().value.origin, 11);
        assert_eq!(row.active().used_abilities, 3);
        assert_eq!(row.object, branch.object);
        assert_eq!(branch.active().value.origin, 12);
        assert!(!row.expire(3, Player::P0, false));
        assert!(row.expire(4, Player::P1, false));
        assert!(row.layers.is_empty());
        assert_eq!(row.physical_card, card);
        assert_eq!(row.original_used, 5);
    }
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
            Zone::Graveyard if !object.is_token() => CopyRecipe::new(object.card),
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
        self.renew_copy_grant_origin(object, true);
        self.refresh_controllers();
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
            self.renew_copy_grant_origin(row.object, false);
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

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};
    /// Bounded core boundary: the three card producers remain gated during this stage.
    #[test]
    fn msh_temporary_copy_bounded_physical_token_identity_and_owned_expiry() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let target = game.object_ref(id);
            let before = game.state.objects[id as usize].clone();
            let recipe = CopyRecipe::new(Card::from_name("@token/msh-moloid").unwrap());
            game.apply_temporary_copy(target, recipe, CopyDeadline::EndOfTurn);
            assert_eq!(game.object_ref(id), target);
            assert!(!game.state.objects[id as usize].is_token());
            assert!(!game.object_view(id).is_token());
            assert_eq!(game.physical_definition(id), before.card);
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            let snapshot = game.snapshot();
            let mut child = game.clone();
            drop(game);
            assert!(child.expire_temporary_copies(true));
            assert_eq!(child.object_ref(id), target);
            assert_eq!(child.state.objects[id as usize], before);
            child.restore(&snapshot).unwrap();
            assert_eq!(child.state.objects[id as usize].card, recipe.definition);
            assert!(!child.state.objects[id as usize].is_token());
            assert_eq!(child.temporary_copy_view(target).unwrap().layers.len(), 1);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_real_token_remains_token_when_copying_physical_card() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "@token/msh-moloid", Zone::Battlefield);
            let target = game.object_ref(id);
            let before = game.state.objects[id as usize].clone();
            game.apply_temporary_copy(
                target,
                CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert!(game.state.objects[id as usize].is_token());
            assert!(game.object_view(id).is_token());
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.state.objects[id as usize], before);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_provider_index_refresh_has_no_entry_event() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let target = game.object_ref(id);
            let entries = game.state.entered_this_turn.clone();
            let pending = game.state.pending_triggers.clone();
            game.apply_temporary_copy(
                target,
                CopyRecipe::new(Card::from_name("The Vision").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(game.state.entered_this_turn, entries);
            assert_eq!(game.state.pending_triggers, pending);
            assert_eq!(
                game.state
                    .cast_sources
                    .iter()
                    .filter(|&&source| source == id)
                    .count(),
                1
            );
            game.apply_temporary_copy(
                target,
                CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
                CopyDeadline::NextTurn {
                    player,
                    created_turn: game.state.turn,
                },
            );
            assert!(!game.state.cast_sources.contains(&id));
            // Removing a hidden older layer does not refresh or rebind the active origin.
            let origin = game.copy_ability_origin(target);
            assert!(!game.expire_temporary_copies(true));
            assert_eq!(game.copy_ability_origin(target), origin);
            assert_eq!(game.object_ref(id), target);
            game.state.turn += 2;
            game.state.active = player;
            assert!(game.expire_temporary_copies(false));
            assert!(!game.state.cast_sources.contains(&id));
            assert_eq!(game.object_ref(id), target);
        }
    }
}

#[cfg(test)]
mod name_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, settle, setup};
    #[test]
    fn msh_temporary_copy_bounded_frozen_name_is_copiable_and_reverts_on_original_only() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
            recipe.overrides = Some(CopyOverrides {
                name: Some("Absorbing Man"),
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            assert_eq!(game.object_view(id).name(), "Absorbing Man");
            assert_eq!(game.object_view(id).card.name(), "Bear Cub");
            assert_eq!(game.physical_definition(id).name(), "Hill Giant");
            let frozen = game.copiable_recipe(object);
            let copies = game.create_copy_groups_inner(
                player,
                &[TokenGroupView {
                    token: frozen.definition,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(frozen)],
                None,
            );
            game.advance_entries();
            settle(&mut game);
            let copy = copies[0];
            assert_eq!(game.object_view(copy.card).name(), "Absorbing Man");
            assert!(game.object_view(copy.card).is_token());
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.object_view(id).name(), "Hill Giant");
            assert_eq!(game.object_view(copy.card).name(), "Absorbing Man");
            assert_eq!(game.object_ref(id), object);
        }
    }
}

#[cfg(test)]
mod type_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, settle, setup};
    #[test]
    fn msh_temporary_copy_bounded_additive_and_replacement_types_survive_copy_of_copy() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            let mut recipe = CopyRecipe::new(Card::from_name("Forest").unwrap());
            recipe.exceptions.base = Some((4, 4));
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(3),
                types: Some(CopyTypes {
                    human_villain: true,
                    ..CopyTypes::default()
                }),
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            let view = game.object_view(id);
            assert_eq!(view.kinds, 3);
            for subtype in ["Forest", "Human", "Villain"] {
                assert!(view.has_subtype(subtype));
            }
            assert_eq!((view.power, view.toughness), (4, 4));
            let mut recipe = CopyRecipe::new(Card::from_name("Namor the Sub-Mariner").unwrap());
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(1),
                types: Some(CopyTypes {
                    replace: true,
                    human_villain: true,
                    mercenary: true,
                    ..CopyTypes::default()
                }),
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            let view = game.object_view(id);
            assert_eq!(view.kinds, 1);
            for subtype in ["Human", "Mercenary", "Villain"] {
                assert!(view.has_subtype(subtype));
            }
            for subtype in ["Merfolk", "Hero", "Forest"] {
                assert!(!view.has_subtype(subtype));
            }
            let recipe = game
                .copiable_recipe(object)
                .with_exceptions(CopyExceptions::GOLEM);
            game.forget_temporary_copy(object);
            let copies = game.create_copy_groups_inner(
                player,
                &[TokenGroupView {
                    token: recipe.definition,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(recipe)],
                None,
            );
            game.advance_entries();
            settle(&mut game);
            let view = game.object_view(copies[0].card);
            assert_eq!(view.kinds, 17);
            for subtype in ["Human", "Mercenary", "Villain", "Golem"] {
                assert!(view.has_subtype(subtype));
            }
            for subtype in ["Merfolk", "Hero"] {
                assert!(!view.has_subtype(subtype));
            }
            assert_eq!((view.power, view.toughness), (3, 3));
            assert!(view.is_token());
        }
    }
}

#[cfg(test)]
mod owned_view_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};
    fn value(game: &Game, viewer: Player) -> serde_json::Value {
        serde_json::to_value(game.observe(viewer)).unwrap()
    }
    #[test]
    fn msh_temporary_copy_bounded_public_blueprint_private_hand_and_orphan_raw_index_restore() {
        for player in [Player::P0, Player::P1] {
            let mut a = setup(player);
            let mut b = setup(player);
            let id = named(&mut a, player, "Hill Giant", Zone::Battlefield);
            assert_eq!(named(&mut b, player, "Hill Giant", Zone::Battlefield), id);
            named(&mut a, player, "Cancel", Zone::Hand);
            named(&mut b, player, "Murder", Zone::Hand);
            assert_eq!(value(&a, player.other()), value(&b, player.other()));
            assert_ne!(value(&a, player), value(&b, player));
            let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
            recipe.overrides = Some(CopyOverrides {
                name: Some("Absorbing Man"),
                ..CopyOverrides::NONE
            });
            for game in [&mut a, &mut b] {
                game.apply_temporary_copy(game.object_ref(id), recipe, CopyDeadline::EndOfTurn);
            }
            assert_eq!(value(&a, player.other()), value(&b, player.other()));
            let view = a.object_view(id);
            assert_eq!(
                view.temporary_copy.as_ref().unwrap().physical_card.name(),
                "Hill Giant"
            );
            assert!(!view.temporary_copy.as_ref().unwrap().physical_token);
            assert_eq!(view.copy_recipe, Some(recipe));
            let captured = serde_json::to_value(&view).unwrap();
            let snapshot = a.snapshot();
            let before = [value(&a, Player::P0), value(&a, Player::P1)];
            let mut raw = a.clone();
            let mut indexed = a.clone();
            drop(a);
            drop(b);
            let actor = raw.actor().unwrap();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|action| *action == Action::Pass)
                .unwrap();
            raw.apply(actor, Action::Pass).unwrap();
            indexed
                .apply_index(actor, indexed.revision(), index)
                .unwrap();
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(value(&raw, viewer), value(&indexed, viewer));
            }
            raw.restore(&snapshot).unwrap();
            for (seat, viewer) in [Player::P0, Player::P1].into_iter().enumerate() {
                let mut expected = before[seat].clone();
                expected["revision"] = raw.revision().into();
                assert_eq!(value(&raw, viewer), expected);
            }
            assert!(raw.expire_temporary_copies(true));
            assert_eq!(raw.object_view(id).name(), "Hill Giant");
            assert_eq!(serde_json::to_value(&view).unwrap(), captured);
            assert!(indexed.object_view(id).temporary_copy.is_some());
            assert!(std::mem::size_of::<GrantedSubtypes>() <= 24);
        }
    }
}

#[cfg(test)]
mod hand_chronology_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
    #[test]
    fn msh_temporary_copy_bounded_hidden_setter_retains_time_against_paid_lasting_wisdom() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("The Ten Rings").unwrap()),
                CopyDeadline::NextTurn {
                    player,
                    created_turn: game.state.turn,
                },
            );
            let older_time = game.copy_hand_time(object).unwrap();
            assert_eq!(game.maximum_hand_size(player), Some(10));
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(game.maximum_hand_size(player), Some(7));
            if game.actor() != Some(player) {
                checked(&mut game, Action::Pass);
            }
            let wisdom = named(&mut game, player, "Wisdom of Ages", Zone::Hand);
            let (colored, generic) = game.state.objects[wisdom as usize].card.cost();
            game.state.players[player.index()].mana = colored;
            game.state.players[player.index()].mana[5] += generic;
            cast(&mut game, wisdom, None, &[]);
            settle(&mut game);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            let lasting = game.state.hand_size.as_ref().unwrap().lasting[player.index()].unwrap();
            assert!(lasting > older_time);
            let snapshot = game.snapshot();
            let mut branch = game.clone();
            drop(game);
            assert!(branch.expire_temporary_copies(true));
            assert_eq!(branch.copy_hand_time(object), Some(older_time));
            assert_eq!(branch.maximum_hand_size(player), None);
            assert_eq!(branch.maximum_hand_size(player.other()), Some(7));
            let timeline = branch.observe(player.other()).hand_size_timeline.unwrap();
            assert_eq!(
                timeline
                    .sources
                    .iter()
                    .find(|row| row.source == object)
                    .unwrap()
                    .timestamp,
                older_time
            );
            branch.restore(&snapshot).unwrap();
            assert!(branch.expire_temporary_copies(true));
            assert_eq!(branch.copy_hand_time(object), Some(older_time));
            assert_eq!(branch.maximum_hand_size(player), None);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_hidden_infinity_stays_before_later_finite_provider() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Niv-Mizzet, Visionary").unwrap()),
                CopyDeadline::NextTurn {
                    player,
                    created_turn: game.state.turn,
                },
            );
            let older_time = game.copy_hand_time(object).unwrap();
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let rings = named(&mut game, player, "The Ten Rings", Zone::Battlefield);
            assert!(game.copy_hand_time(game.object_ref(rings)).unwrap() > older_time);
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.copy_hand_time(object), Some(older_time));
            assert_eq!(game.maximum_hand_size(player), Some(10));
            assert_eq!(game.object_ref(id), object);
        }
    }
}

#[cfg(test)]
mod characteristic_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
    #[test]
    fn msh_temporary_copy_bounded_later_copy_obeys_earlier_removal_and_independent_grant_order() {
        for player in [Player::P0, Player::P1] {
            for remove_provider in [false, true] {
                let mut game = setup(player);
                let provider = named(&mut game, player, "Hill Giant", Zone::Battlefield);
                let recipient = named(&mut game, player, "Bear Cub", Zone::Battlefield);
                let aura = named(&mut game, player, "Witness Protection", Zone::Hand);
                game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
                cast(
                    &mut game,
                    aura,
                    None,
                    &[Target::Card(if remove_provider {
                        provider
                    } else {
                        recipient
                    })],
                );
                settle(&mut game);
                game.apply_temporary_copy(
                    game.object_ref(provider),
                    CopyRecipe::new(Card::from_name("Aggressive Mammoth").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
                assert_eq!(
                    game.keywords(provider).contains(Keywords::TRAMPLE),
                    !remove_provider
                );
                assert_eq!(
                    game.keywords(recipient).contains(Keywords::TRAMPLE),
                    !remove_provider
                );
                assert_eq!(game.printed_abilities_available(provider), !remove_provider);
                game.expire_temporary_copies(true);
                assert!(!game.keywords(recipient).contains(Keywords::TRAMPLE));
            }
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_copied_intrinsic_vigilance_and_fixed_pt_obey_paid_suppression() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            let mut recipe = CopyRecipe::new(Card::from_name("Namor the Sub-Mariner").unwrap());
            recipe.exceptions.base = Some((4, 4));
            recipe.overrides = Some(CopyOverrides {
                keywords: Keywords::VIGILANCE,
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert!(
                game.keywords(id)
                    .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
            );
            if game.actor() != Some(player) {
                checked(&mut game, Action::Pass);
            }
            let aura = named(&mut game, player, "Witness Protection", Zone::Hand);
            game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
            cast(&mut game, aura, None, &[Target::Card(id)]);
            settle(&mut game);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert!(!game.keywords(id).contains(Keywords::VIGILANCE));
            assert!(!game.keywords(id).contains(Keywords::FLYING));
            assert_eq!(game.copiable_recipe(object), recipe);
            game.move_object(aura, Zone::Graveyard);
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert!(game.keywords(id).contains(Keywords::VIGILANCE));
            assert_eq!(game.object_ref(id), object);
            game.expire_temporary_copies(true);
            assert_eq!(
                game.stats(id),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert!(!game.keywords(id).contains(Keywords::VIGILANCE));
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_legend_exceptions_drive_target_legality_and_sba_without_card_mutation()
     {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let a = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let b = named(&mut game, player, "Bear Cub", Zone::Battlefield);
            let mut recipe = CopyRecipe::new(Card::from_name("Niv-Mizzet, Visionary").unwrap());
            recipe.overrides = Some(CopyOverrides {
                legendary: Some(false),
                ..CopyOverrides::NONE
            });
            for id in [a, b] {
                game.apply_temporary_copy(game.object_ref(id), recipe, CopyDeadline::EndOfTurn);
            }
            assert!(game.state.objects[a as usize].card.is_legendary());
            assert!(!game.object_view(a).is_legendary());
            assert!(!game.sba_required());
            const LEGEND: TargetRule = TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Legendary,
            };
            assert!(!game.cast_target_matches(LEGEND, player, game.object_ref(b), Target::Card(a)));
            recipe.overrides.as_mut().unwrap().legendary = Some(true);
            for id in [a, b] {
                game.apply_temporary_copy(game.object_ref(id), recipe, CopyDeadline::EndOfTurn);
            }
            assert!(game.object_view(a).is_legendary());
            assert!(game.cast_target_matches(LEGEND, player, game.object_ref(b), Target::Card(a)));
            assert!(game.sba_required());
            let frozen = game.object_view(a);
            game.expire_temporary_copies(true);
            assert!(!game.object_view(a).is_legendary());
            assert!(frozen.is_legendary());
            assert!(!game.sba_required());
        }
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
    fn paid(game: &mut Game, player: Player, name: &str, targets: &[Target]) {
        if game.actor() != Some(player) {
            checked(game, Action::Pass);
        }
        let spell = named(game, player, name, Zone::Hand);
        let (colored, generic) = game.state.objects[spell as usize].card.cost();
        game.state.players[player.index()].mana = colored;
        game.state.players[player.index()].mana[5] += generic;
        cast(game, spell, None, targets);
        settle(game);
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_murder_preserves_real_card_and_real_token_departures() {
        for player in [Player::P0, Player::P1] {
            for token in [false, true] {
                let mut game = setup(player);
                let id = named(
                    &mut game,
                    player,
                    if token {
                        "@token/msh-moloid"
                    } else {
                        "Hill Giant"
                    },
                    Zone::Battlefield,
                );
                let original = game.state.objects[id as usize].card;
                let object = game.object_ref(id);
                game.apply_temporary_copy(
                    object,
                    CopyRecipe::new(
                        Card::from_name(if token {
                            "Hill Giant"
                        } else {
                            "@token/msh-moloid"
                        })
                        .unwrap(),
                    ),
                    CopyDeadline::EndOfTurn,
                );
                let before = game.object_view(id);
                assert_eq!(before.is_token(), token);
                paid(&mut game, player, "Murder", &[Target::Card(id)]);
                assert_eq!(game.state.objects[id as usize].card, original);
                assert_eq!(game.state.objects[id as usize].is_token(), token);
                assert_eq!(
                    game.state.objects[id as usize].zone,
                    if token {
                        Zone::Outside
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_ne!(game.object_ref(id), object);
                assert!(game.state.temporary_copies.is_none());
                assert!(game.copy_recipe(object).is_none());
                assert!(before.temporary_copy.is_some());
                assert_eq!(before.is_token(), token);
            }
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_actual_cleanup_and_captured_next_turn_use_existing_identity() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let eot = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let next = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let refs = [game.object_ref(eot), game.object_ref(next)];
            for (object, deadline) in [
                (refs[0], CopyDeadline::EndOfTurn),
                (
                    refs[1],
                    CopyDeadline::NextTurn {
                        player,
                        created_turn: game.state.turn,
                    },
                ),
            ] {
                game.apply_temporary_copy(
                    object,
                    CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                    deadline,
                );
            }
            // Control changes do not change the player named by the duration.
            game.state.objects[next as usize].controller = player.other();
            next_turn(&mut game);
            assert_eq!(game.state.active, player.other());
            assert_eq!(game.object_view(eot).name(), "Hill Giant");
            assert_eq!(game.object_view(next).name(), "Bear Cub");
            assert_eq!(game.object_ref(eot), refs[0]);
            assert_eq!(game.object_ref(next), refs[1]);
            next_turn(&mut game);
            assert_eq!(game.state.active, player);
            assert_eq!(game.object_view(next).name(), "Hill Giant");
            assert_eq!(game.object_ref(next), refs[1]);
            assert_eq!(game.state.objects[next as usize].controller, player.other());
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_cleanup_copy_expiry_causes_sba_before_turn_advance() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Bear Cub", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            game.state.objects[id as usize].counters.minus_one = 2;
            assert_eq!(game.stats(id).toughness, 1);
            game.state.step = Step::End;
            let turn = game.state.turn;
            checked(&mut game, Action::Pass);
            checked(&mut game, Action::Pass);
            assert_eq!(game.state.turn, turn);
            assert_eq!(game.state.step, Step::Cleanup);
            assert_eq!(game.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(game.state.objects[id as usize].card.name(), "Bear Cub");
            assert_ne!(game.object_ref(id), object);
            assert!(game.state.temporary_copies.is_none());
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_time_stop_uses_same_eot_expiry_boundary() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let turn = game.state.turn;
            paid(&mut game, player, "Time Stop", &[]);
            assert!(game.state.turn > turn);
            assert_eq!(game.object_view(id).name(), "Hill Giant");
            assert_eq!(game.object_ref(id), object);
            assert!(game.state.temporary_copies.is_none());
        }
    }
}

#[cfg(test)]
mod modal_origin_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{checked, named, settle, setup};
    pub(super) fn announce(game: &mut Game, player: Player, name: &str, targets: &[Target]) {
        if game.actor() != Some(player) {
            checked(game, Action::Pass);
        }
        let id = named(game, player, name, Zone::Hand);
        let (colored, generic) = game.state.objects[id as usize].card.cost();
        game.state.players[player.index()].mana = colored;
        game.state.players[player.index()].mana[5] += generic;
        checked(game, Action::AnnounceCast { card: id });
        for &target in targets {
            checked(game, Action::ChooseTarget { target });
        }
        while game
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.card == id && a.item.kind == StackKind::Spell)
        {
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(game, action);
        }
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
    pub(super) fn modes(game: &mut Game) -> Vec<u16> {
        game.prepare_triggers();
        for _ in 0..12 {
            if matches!(game.state.pending, Pending::Modes { .. }) {
                return game
                    .legal_actions()
                    .into_iter()
                    .filter_map(|a| {
                        if let Action::ChooseMode { mode } = a {
                            Some(mode)
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(game, action);
        }
        panic!("copied Vision trigger did not offer modes");
    }
    pub(super) fn choose(game: &mut Game, mode: u16) {
        assert!(modes(game).contains(&mode));
        checked(game, Action::ChooseMode { mode });
        settle(game);
    }
    pub(super) fn shock(game: &mut Game, player: Player) {
        announce(game, player, "Shock", &[Target::Player(player.other())]);
    }
    fn copy_vision(game: &mut Game, id: CardId, deadline: CopyDeadline) -> u64 {
        let object = game.object_ref(id);
        game.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::from_name("The Vision").unwrap()),
            deadline,
        );
        game.copy_ability_origin(object).unwrap()
    }
    /// Primary-backed inference: CR113.2c/707.2 and the official Likeness Looter
    /// new-instance ruling imply a fresh acquisition on recopying identical text.
    /// CR602.5c is not itself applied to Vision's triggered ability.
    #[test]
    fn msh_temporary_copy_bounded_identical_vision_recopy_has_new_modes_then_restores_older_history()
     {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            let turn = game.state.turn;
            let older = copy_vision(
                &mut game,
                id,
                CopyDeadline::NextTurn {
                    player,
                    created_turn: turn,
                },
            );
            shock(&mut game, player);
            choose(&mut game, 0);
            let newer = copy_vision(&mut game, id, CopyDeadline::EndOfTurn);
            assert_ne!(older, newer);
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![0, 1, 2]);
            choose(&mut game, 0);
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.copy_ability_origin(object), Some(older));
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![1, 2]);
            let snapshot = game.snapshot();
            let mut child = game.clone();
            drop(game);
            choose(&mut child, 1);
            child.restore(&snapshot).unwrap();
            assert_eq!(modes(&mut child), vec![1, 2]);
            choose(&mut child, 2);
            assert_eq!(child.object_ref(id), object);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_printed_vision_history_returns_with_its_original_instance() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "The Vision", Zone::Battlefield);
            shock(&mut game, player);
            choose(&mut game, 0);
            copy_vision(&mut game, id, CopyDeadline::EndOfTurn);
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![0, 1, 2]);
            choose(&mut game, 1);
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.copy_ability_origin(game.object_ref(id)), None);
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![1, 2]);
            choose(&mut game, 1);
            assert!(
                game.state
                    .modal_history
                    .as_ref()
                    .unwrap()
                    .iter()
                    .all(|row| row.origin.is_none())
            );
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_removal_and_control_change_preserve_vision_origin() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            let turn = game.state.turn;
            let origin = copy_vision(
                &mut game,
                id,
                CopyDeadline::NextTurn {
                    player,
                    created_turn: turn,
                },
            );
            shock(&mut game, player);
            choose(&mut game, 0);
            announce(&mut game, player, "Witness Protection", &[Target::Card(id)]);
            let aura = game
                .state
                .stack
                .iter()
                .find(|item| item.kind == StackKind::Spell)
                .unwrap()
                .card;
            choose(&mut game, 2);
            assert!(!game.printed_abilities_available(id));
            shock(&mut game, player);
            settle(&mut game);
            assert_eq!(game.copy_ability_origin(object), Some(origin));
            game.move_object(aura, Zone::Graveyard);
            assert!(game.printed_abilities_available(id));
            game.state.objects[id as usize].controller = player.other();
            assert_eq!(game.copy_ability_origin(object), Some(origin));
            shock(&mut game, player.other());
            assert_eq!(modes(&mut game), vec![1]);
            choose(&mut game, 1);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_queued_vision_packet_retains_old_origin_after_identical_recopy() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let turn = game.state.turn;
            let old = copy_vision(
                &mut game,
                id,
                CopyDeadline::NextTurn {
                    player,
                    created_turn: turn,
                },
            );
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![0, 1, 2]);
            assert_eq!(
                game.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .source_ability_origin,
                Some(old)
            );
            let new = copy_vision(&mut game, id, CopyDeadline::EndOfTurn);
            assert_ne!(new, old);
            choose(&mut game, 0);
            assert!(
                game.state
                    .modal_history
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|row| row.origin == Some(old) && row.modes == [0])
            );
            shock(&mut game, player);
            assert_eq!(modes(&mut game), vec![0, 1, 2]);
            choose(&mut game, 0);
        }
    }
}

#[cfg(test)]
mod restricted_origin_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
    fn activate(game: &mut Game, player: Player, id: CardId) {
        if game.actor() != Some(player) {
            checked(game, Action::Pass);
        }
        game.state.players[player.index()].mana = [20; 6];
        checked(
            game,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for _ in 0..8 {
            if game.state.announcing.is_none() {
                settle(game);
                return;
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(game, action);
        }
        panic!("copied activation remained unpaid");
    }
    fn can_activate(game: &Game, id: CardId) -> bool {
        game.legal_actions().contains(&Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        })
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_power_up_and_exhaust_belong_to_independent_origins() {
        for player in [Player::P0, Player::P1] {
            for definition in ["Aerial Doombot", "Mild-Mannered Librarian"] {
                let mut game = setup(player);
                let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
                let object = game.object_ref(id);
                let recipe = CopyRecipe::new(Card::from_name(definition).unwrap());
                game.apply_temporary_copy(
                    object,
                    recipe,
                    CopyDeadline::NextTurn {
                        player,
                        created_turn: game.state.turn,
                    },
                );
                let old = game.copy_ability_origin(object);
                activate(&mut game, player, id);
                if game.actor() != Some(player) {
                    checked(&mut game, Action::Pass);
                }
                assert!(!can_activate(&game, id));
                game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
                let new = game.copy_ability_origin(object);
                assert_ne!(old, new);
                game.state.players[player.index()].mana = [20; 6];
                assert!(can_activate(&game, id));
                activate(&mut game, player, id);
                if definition == "Aerial Doombot" {
                    let uses = game.object_view(id).power_up_uses;
                    assert_eq!(uses.len(), 2);
                    assert!(uses.iter().all(|row| row.count == 1));
                    assert!(uses.iter().any(|row| row.origin == old));
                    assert!(uses.iter().any(|row| row.origin == new));
                }
                assert!(game.expire_temporary_copies(true));
                if game.actor() != Some(player) {
                    checked(&mut game, Action::Pass);
                }
                assert_eq!(game.copy_ability_origin(object), old);
                assert!(!can_activate(&game, id));
                // Lifetime Power-up/exhaust restrictions survive a turn boundary.
                next_turn(&mut game);
                assert_eq!(game.object_ref(id), object);
                if definition == "Mild-Mannered Librarian" {
                    assert_eq!(game.state.objects[id as usize].used_abilities & 1, 1);
                }
            }
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_counter_triggers_limit_each_acquisition_then_restore_old_use()
     {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let bear = named(&mut game, player, "Bear Cub", Zone::Battlefield);
            let object = game.object_ref(id);
            let recipe = CopyRecipe::new(Card::from_name("Knight of Wundagore").unwrap());
            game.apply_temporary_copy(
                object,
                recipe,
                CopyDeadline::NextTurn {
                    player,
                    created_turn: game.state.turn,
                },
            );
            let old = game.copy_ability_origin(object);
            for stage in 0..4 {
                if stage == 2 {
                    game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
                }
                if stage == 3 {
                    assert!(game.expire_temporary_copies(true));
                }
                if game.actor() != Some(player) {
                    checked(&mut game, Action::Pass);
                }
                let veil = named(&mut game, player, "Snakeskin Veil", Zone::Hand);
                game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
                cast(&mut game, veil, None, &[Target::Card(bear)]);
                settle(&mut game);
                assert_eq!(
                    game.counter_count(id, CounterKind::PlusOne),
                    if stage < 2 { 1 } else { 2 }
                );
            }
            assert_eq!(game.copy_ability_origin(object), old);
            let uses: Vec<_> = game
                .state
                .triggered_this_turn
                .iter()
                .filter(|row| row.source == object)
                .collect();
            assert_eq!(uses.len(), 2);
            assert_ne!(uses[0].origin, uses[1].origin);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_linked_entry_choice_is_preserved_as_fact_but_not_inherited() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Sorcerous Spyglass", Zone::Battlefield);
            let object = game.object_ref(id);
            let choice =
                EntryChoiceValue::CardName(crate::CardName::from_name("Arc Reactor").unwrap());
            // Bounded prior-entry fixture; acquisition and restoration below use
            // the same physical incarnation and its actual stored choice.
            game.set_entry_value(object, choice);
            assert!(game.named_activation_prohibited_for_name("Arc Reactor"));
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Sorcerous Spyglass").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(game.object_view(id).entry_choice, Some(choice));
            assert_eq!(
                ability_world::CharacteristicWorld::entry_value(&game, object),
                None
            );
            assert!(!game.named_activation_prohibited_for_name("Arc Reactor"));
            let frozen = game.object_view(id);
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.object_view(id).entry_choice, Some(choice));
            assert!(game.named_activation_prohibited_for_name("Arc Reactor"));
            assert_eq!(frozen.entry_choice, Some(choice));
        }
    }
}

#[cfg(test)]
mod timestamp_face_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
    fn announce_activation(game: &mut Game, player: Player, id: CardId, target: Option<CardId>) {
        if game.actor() != Some(player) {
            checked(game, Action::Pass);
        }
        game.state.players[player.index()].mana = [20; 6];
        checked(
            game,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if let Some(target) = target {
            checked(
                game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        for _ in 0..8 {
            if game.state.announcing.is_none() {
                return;
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(game, action);
        }
        panic!("copied timestamp activation remained unpaid");
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_equipping_retimes_hidden_setter_without_new_acquisition() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let bear = named(&mut game, player, "Bear Cub", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("The Ten Rings").unwrap()),
                CopyDeadline::NextTurn {
                    player,
                    created_turn: game.state.turn,
                },
            );
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Hawkeye's Bow").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let before = game.temporary_copy_view(object).unwrap();
            if game.actor() != Some(player) {
                checked(&mut game, Action::Pass);
            }
            let wisdom = named(&mut game, player, "Wisdom of Ages", Zone::Hand);
            let (colored, generic) = game.state.objects[wisdom as usize].card.cost();
            game.state.players[player.index()].mana = colored;
            game.state.players[player.index()].mana[5] += generic;
            cast(&mut game, wisdom, None, &[]);
            settle(&mut game);
            let lasting = game.state.hand_size.as_ref().unwrap().lasting[player.index()].unwrap();
            assert!(before.layers[0].hand_timestamp.unwrap() < lasting);
            announce_activation(&mut game, player, id, Some(bear));
            settle(&mut game);
            assert_eq!(
                game.state.objects[id as usize].attached_to,
                Some(game.object_ref(bear))
            );
            let after = game.temporary_copy_view(object).unwrap();
            for (old, new) in before.layers.iter().zip(&after.layers) {
                assert_eq!(old.origin, new.origin);
                assert!(new.provider_timestamp > old.provider_timestamp);
            }
            assert!(after.layers[0].hand_timestamp.unwrap() > lasting);
            assert!(game.expire_temporary_copies(true));
            assert_eq!(game.maximum_hand_size(player), Some(10));
            assert_eq!(game.copy_hand_time(object), after.layers[0].hand_timestamp);
            assert_eq!(
                game.copy_ability_origin(object),
                Some(before.layers[0].origin)
            );
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_queued_transform_changes_physical_face_beneath_copy_only() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Monica Rambeau", Zone::Battlefield);
            let object = game.object_ref(id);
            announce_activation(&mut game, player, id, None);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let before = game.temporary_copy_view(object).unwrap();
            settle(&mut game);
            let after = game.temporary_copy_view(object).unwrap();
            assert_eq!(after.active_origin, before.active_origin);
            assert!(after.layers[0].provider_timestamp > before.layers[0].provider_timestamp);
            assert_eq!(game.object_view(id).name(), "Hill Giant");
            let face = game.object_view(id).face.unwrap();
            assert!(face.back);
            assert_eq!(face.transforms, 1);
            assert_eq!(game.physical_definition(id).name(), "Photon, Living Light");
            game.expire_temporary_copies(true);
            assert_eq!(game.object_view(id).name(), "Photon, Living Light");
            assert_eq!(game.object_ref(id), object);
        }
    }
    #[test]
    fn msh_temporary_copy_bounded_paid_copied_transform_cannot_turn_a_single_faced_card_over() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
            let object = game.object_ref(id);
            game.apply_temporary_copy(
                object,
                CopyRecipe::new(Card::from_name("Monica Rambeau").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let before = game.temporary_copy_view(object).unwrap();
            announce_activation(&mut game, player, id, None);
            settle(&mut game);
            assert_eq!(game.temporary_copy_view(object), Some(before));
            assert_eq!(game.object_view(id).name(), "Monica Rambeau");
            assert!(game.object_view(id).face.is_none());
            assert_eq!(game.transform_serial(object), 0);
            game.expire_temporary_copies(true);
            assert_eq!(game.object_view(id).name(), "Hill Giant");
        }
    }
}

#[cfg(test)]
#[path = "engine_temporary_copy_dispatch_tests.rs"]
mod dispatch_tests;
#[cfg(test)]
#[path = "engine_temporary_copy_grant_tests.rs"]
mod grant_tests;
