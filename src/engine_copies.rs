//! Runtime-created token recipes. Immutable definitions stay shared; copying a copy
//! flattens to its existing recipe, never to a pointer into another game object.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyRecipe {
    /// The complete immutable copiable definition, including fixed token exceptions.
    pub definition: Card,
    pub exceptions: CopyExceptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<CopyOverrides>,
    // Physical token face structure is independent of the active copiable
    // definition (CR707.8a). Ordinary recipes derive it from their definition.
    #[serde(skip)]
    pub(super) faces: Option<CopyFaces>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CopyFaces {
    Single,
    Double {
        physical_front: Card,
        front: Card,
        back: Card,
        back_up: bool,
    },
}
impl CopyFaces {
    fn for_definition(definition: Card) -> Self {
        match definition.other_face() {
            Some(_) => Self::Double {
                physical_front: definition.front_face(),
                front: definition.front_face(),
                back: definition.front_face().other_face().unwrap(),
                back_up: definition.face_parent().is_some(),
            },
            None => Self::Single,
        }
    }
    pub(super) fn turned(self) -> Option<(Self, Card)> {
        match self {
            Self::Single => None,
            Self::Double {
                physical_front,
                front,
                back,
                back_up,
            } => Some((
                Self::Double {
                    physical_front,
                    front,
                    back,
                    back_up: !back_up,
                },
                if back_up { front } else { back },
            )),
        }
    }
}
impl CopyRecipe {
    pub(super) fn face_structure(self) -> CopyFaces {
        self.faces
            .unwrap_or_else(|| CopyFaces::for_definition(self.definition))
    }
    pub(super) fn with_faces(mut self, faces: CopyFaces) -> Self {
        self.faces = (faces != CopyFaces::for_definition(self.definition)).then_some(faces);
        self
    }
    pub(super) fn with_copy_effect_faces(self, mut faces: CopyFaces) -> Self {
        if let CopyFaces::Double { front, back, .. } = &mut faces {
            // A copy effect applies to the permanent, including both faces.
            *front = self.definition;
            *back = self.definition;
        }
        self.with_faces(faces)
    }
    pub(super) fn turn_face(&mut self) -> Option<Card> {
        let (faces, definition) = self.face_structure().turned()?;
        self.definition = definition;
        *self = self.with_faces(faces);
        Some(definition)
    }
    /// CR707.9d: a replacement subtype set excludes the copied subtype CDA.
    /// Later copied exceptions and external keyword grants are separate effects.
    pub(super) fn intrinsic_keywords(self) -> Keywords {
        let keywords = self.definition.keywords();
        if self.copy_types().replace {
            keywords.without(Keywords::CHANGELING)
        } else {
            keywords
        }
    }
    pub(super) fn kinds(self) -> u16 {
        let mut kinds = self.definition.kind_mask();
        if self.exceptions.artifact {
            kinds |= 16;
        }
        if self.exceptions.golem {
            kinds |= 17;
        }
        if self.exceptions.fractal {
            kinds |= 1;
        }
        self.overrides
            .and_then(|value| value.kinds)
            .unwrap_or(kinds)
    }
    pub(super) fn copy_types(self) -> CopyTypes {
        self.overrides
            .and_then(|value| value.types)
            .unwrap_or(CopyTypes {
                nightmare: self.exceptions.nightmare,
                golem: self.exceptions.golem,
                fractal: self.exceptions.fractal,
                ..CopyTypes::default()
            })
    }
    /// A later copy exception is applied after the already flattened blueprint.
    /// Keep ordinary preexisting recipes in their original representation.
    pub(super) fn with_exceptions(mut self, exceptions: CopyExceptions) -> Self {
        if let Some(overrides) = &mut self.overrides {
            if let Some(kinds) = &mut overrides.kinds {
                if exceptions.artifact {
                    *kinds |= 16;
                }
                if exceptions.golem {
                    *kinds |= 17;
                }
                if exceptions.fractal {
                    *kinds |= 1;
                }
            }
            if let Some(types) = &mut overrides.types {
                types.nightmare |= exceptions.nightmare;
                types.golem |= exceptions.golem;
                types.fractal |= exceptions.fractal;
            }
        }
        self.exceptions = self.exceptions.union(exceptions);
        self
    }
    pub const fn new(definition: Card) -> Self {
        Self {
            definition,
            exceptions: CopyExceptions::NONE,
            overrides: None,
            faces: None,
        }
    }

    pub const fn factory_identity(self) -> &'static str {
        "Copy"
    }
    pub(super) fn token_info(self) -> TokenInfo {
        let card = self.definition;
        TokenInfo {
            factory_identity: self.factory_identity(),
            name: self
                .overrides
                .and_then(|v| v.name)
                .unwrap_or_else(|| card.name()),
            subtypes: card
                .info()
                .type_line
                .split_once('—')
                .map_or("", |(_, types)| types.trim()),
            colors: card.colors(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CopyRow {
    pub object: ObjectRef,
    pub recipe: CopyRecipe,
}
pub(super) type Rows = Option<SharedVec<CopyRow>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CopyRow>>() + rows.len() * std::mem::size_of::<CopyRow>()
    })
}
impl Game {
    fn copy_recipe_index(&self, object: ObjectRef) -> Option<usize> {
        let rows = self.state.copy_recipes.as_ref()?;
        let index = rows
            .binary_search_by_key(&object.card, |row| row.object.card)
            .ok()?;
        (rows[index].object == object).then_some(index)
    }
    pub(super) fn copy_recipe(&self, object: ObjectRef) -> Option<CopyRecipe> {
        let index = self.copy_recipe_index(object)?;
        Some(self.state.copy_recipes.as_ref().unwrap()[index].recipe)
    }
    pub(super) fn set_copy_recipe(&mut self, object: ObjectRef, recipe: CopyRecipe) {
        assert_eq!(self.object_ref(object.card), object);
        self.clear_copy_recipe(object);
        let rows = self.state.copy_recipes.get_or_insert_default();
        rows.try_reserve(1)
            .capacity("copy recipe allocation exhausted");
        let index = rows.partition_point(|row| row.object.card < object.card);
        rows.insert(index, CopyRow { object, recipe });
    }
    pub(super) fn clear_copy_recipe(&mut self, object: ObjectRef) {
        let Some(index) = self.copy_recipe_index(object) else {
            return;
        };
        let rows = self.state.copy_recipes.as_mut().unwrap();
        rows.remove(index);
        if rows.is_empty() {
            self.state.copy_recipes = None;
        }
    }
    pub(super) fn advance_copy_recipe(&mut self, before: ObjectRef, after: ObjectRef) {
        let Some(index) = self.copy_recipe_index(before) else {
            return;
        };
        assert_eq!(before.card, after.card);
        self.state.copy_recipes.as_mut().unwrap()[index].object = after;
    }
    pub(super) fn release_unentered_copy(&mut self, object: ObjectRef) {
        self.clear_copy_recipe(object);
        let slot = &mut self.state.objects[object.card as usize];
        assert_eq!(slot.zone, Zone::Outside);
        slot.generation = slot
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        self.state.free_tokens.push(object.card);
    }
    pub(super) fn create_copy_groups_inner(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
        temporary: Option<&SpellView>,
    ) -> Vec<ObjectRef> {
        assert_eq!(groups.len(), recipes.len());
        let total = groups
            .iter()
            .try_fold(0u128, |total, g| total.checked_add(g.amount))
            .capacity("token quantity exhausted");
        let amount = usize::try_from(total).capacity("token count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&end| end <= u128::from(CardId::MAX) + 1)
            .capacity("object slots exhausted");
        self.state
            .objects
            .try_reserve(fresh)
            .capacity("object allocation exhausted");
        let mut moves = Vec::new();
        moves
            .try_reserve_exact(amount)
            .capacity("token batch allocation exhausted");
        for (group, recipe) in groups.iter().zip(recipes) {
            assert!(recipe.is_some() || group.token.is_token());
            for _ in 0..group.amount {
                let (id, generation) = if let Some(slot) = self.replay_frozen_mana_copy_slot() {
                    slot
                } else if let Some(id) = self.state.free_tokens.pop() {
                    let old = &self.state.objects[id as usize];
                    assert!((old.is_token() || old.is_spell_copy()) && old.zone == Zone::Outside);
                    (id, old.generation)
                } else {
                    (
                        CardId::try_from(self.state.objects.len())
                            .capacity("object slots exhausted"),
                        0,
                    )
                };
                self.record_cast_mana_allocation(id, generation);
                let object = Object {
                    card: group.token,
                    copy_kind: if recipe.is_some() {
                        CopyKind::Token
                    } else {
                        CopyKind::None
                    },
                    owner: player,
                    controller: player,
                    zone: Zone::Outside,
                    generation,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    cast_for_warp: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    dealt_combat_damage_this_incarnation: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    loyalty_activated_this_turn: false,
                    class_level: 1,
                    attached_to: None,
                };
                if id as usize == self.state.objects.len() {
                    self.state.objects.push(object);
                } else {
                    self.state.objects[id as usize] = object;
                }
                if let Some(recipe) = recipe {
                    let reference = self.object_ref(id);
                    let rows = self.state.copy_recipes.get_or_insert_with(Default::default);
                    rows.try_reserve(1)
                        .capacity("copy recipe allocation exhausted");
                    let index = rows.partition_point(|row| row.object.card < id);
                    rows.insert(
                        index,
                        CopyRow {
                            object: reference,
                            recipe: *recipe,
                        },
                    );
                }
                moves.push(self.entry_move(id, Zone::Battlefield, group.tapped));
            }
        }
        let created = moves
            .iter()
            .map(|m| ObjectRef {
                card: m.before.id,
                generation: m
                    .before
                    .reference
                    .generation
                    .checked_add(1)
                    .capacity("object generation exhausted"),
            })
            .collect();
        if !moves.is_empty() {
            self.track_nightmare_copy_entries(&moves);
            if let Some(item) = temporary {
                self.stage_temporary_copy_entries(moves, item);
            } else {
                self.stage_entries(moves, None);
            }
        }
        created
    }
}
