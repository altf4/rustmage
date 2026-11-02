//! Owned non-targeted as-entry copy choices; physical objects remain unchanged until commit.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Preparation {
    pub done: bool,
    pub recipe: Option<CopyRecipe>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryCopyChoiceView {
    pub entrant: ObjectRef,
    pub card: Card,
    pub optional: bool,
    pub candidates: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn needs_copy(movement: &entries::EntryMove) -> bool {
    movement.destination == Zone::Battlefield
        && movement.before.zone != Zone::Battlefield
        && !movement.skipped
        && movement.manifest_original.is_none()
        && movement.before.card.entry_copy().is_some()
        && movement
            .copy_preparation
            .as_ref()
            .is_none_or(|copy| !copy.done)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn apply_blueprint(movement: &mut entries::EntryMove, recipe: CopyRecipe) {
    if movement.source_before.is_none() {
        movement.source_before = Some(Box::new(movement.before.clone()));
    }
    movement.before.card = recipe.definition;
    movement.before.name = recipe
        .overrides
        .and_then(|o| o.name)
        .unwrap_or_else(|| recipe.definition.name());
    movement.before.kinds = recipe.kinds();
    movement.before.colors = recipe.definition.colors();
    movement.before.keywords = recipe
        .intrinsic_keywords()
        .union(recipe.overrides.map_or(Keywords::NONE, |o| o.keywords));
    movement.before.copy_recipe = Some(recipe);
    movement.choice = None;
    movement.life_paid = None;
    movement.attachment = None;
    movement.before.granted_subtypes = Game::entry_replacement_subtypes(movement);
    let base = recipe
        .exceptions
        .base
        .map(|(p, t)| (i128::from(p), i128::from(t)))
        .unwrap_or_else(|| match recipe.definition.effect() {
            Effect::Creature(p, t) => (i128::from(p), i128::from(t)),
            _ => (0, 0),
        });
    movement.before.power = base.0;
    movement.before.toughness = base.1;
    movement.copy_preparation = Some(Box::new(Preparation {
        done: false,
        recipe: Some(recipe),
    }));
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn current_entry_copy_index(&self) -> Option<usize> {
        let batch = self.state.entry_batch.as_ref()?;
        batch
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| needs_copy(m))
            .min_by_key(|(index, m)| (m.controller != self.state.active, *index))
            .map(|(index, _)| index)
    }
    pub(super) fn entry_copy_candidates(&self) -> Vec<ObjectRef> {
        let Some(index) = self.current_entry_copy_index() else {
            return Vec::new();
        };
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[index];
        let spec = movement.before.card.entry_copy().unwrap();
        self.state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, _)| {
                let id = id as CardId;
                (!batch
                    .moves
                    .iter()
                    .any(|m| m.before.id == id && m.before.zone != Zone::Battlefield)
                    && spec.filters.iter().any(|&filter| {
                        self.battlefield_filter_matches(filter, movement.controller, id)
                    }))
                .then(|| self.object_ref(id))
            })
            .collect()
    }
    pub(super) fn advance_entry_copies(&mut self) -> bool {
        loop {
            let Some(index) = self.current_entry_copy_index() else {
                return false;
            };
            if !self.entry_copy_candidates().is_empty() {
                let player = self.state.entry_batch.as_ref().unwrap().moves[index].controller;
                self.state.pending = Pending::EntryCopy { player };
                return true;
            }
            self.state.entry_batch.as_mut().unwrap().moves[index]
                .copy_preparation
                .get_or_insert_default()
                .done = true;
        }
    }
    pub(super) fn choose_entry_copy(&mut self, object: Option<ObjectRef>) {
        let index = self.current_entry_copy_index().unwrap();
        if let Some(object) = object {
            let recipe = self.copiable_recipe(object);
            apply_blueprint(
                &mut self.state.entry_batch.as_mut().unwrap().moves[index],
                recipe,
            );
        } else {
            self.state.entry_batch.as_mut().unwrap().moves[index]
                .copy_preparation
                .get_or_insert_default()
                .done = true;
        }
    }
    pub(super) fn entry_copy_view(&self, viewer: Player) -> Option<EntryCopyChoiceView> {
        if !matches!(self.state.pending, Pending::EntryCopy { .. }) {
            return None;
        }
        let index = self.current_entry_copy_index()?;
        let movement = &self.state.entry_batch.as_ref()?.moves[index];
        if !entries::visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        Some(EntryCopyChoiceView {
            entrant: movement.before.reference,
            card: movement
                .source_before
                .as_ref()
                .map_or(movement.before.card, |v| v.card),
            optional: movement.before.card.entry_copy()?.optional,
            candidates: self.entry_copy_candidates(),
        })
    }
}

/// Flat layer-one blueprint in an actual or projected object world.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn blueprint_in_world<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> CopyRecipe {
    if world.face_down(id) {
        return CopyRecipe::new(world.definition(id));
    }
    let recipe = world.recipe(id);
    if let Some(row) = world
        .temporary_copy_rows_state()
        .as_ref()
        .and_then(|rows| rows.iter().find(|row| row.object == world.reference(id)))
    {
        let physical = row
            .original_recipe
            .unwrap_or_else(|| CopyRecipe::new(row.physical_card));
        recipe.with_copy_effect_faces(physical.face_structure())
    } else {
        recipe
    }
}
