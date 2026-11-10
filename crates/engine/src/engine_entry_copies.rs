//! Owned non-targeted as-entry copy choices; physical objects remain unchanged until commit.
use super::*;
use crate::cards::EntryCopySource;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EntryCopyExile {
    definition: Card,
    ability: u16,
    object: ObjectRef,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Preparation {
    pub done: bool,
    pub recipe: Option<CopyRecipe>,
    pub exile: Option<EntryCopyExile>,
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
        && movement
            .intrinsic_entry_card()
            .is_some_and(|card| card.entry_copy().is_some())
        && movement
            .copy_preparation
            .as_ref()
            .is_none_or(|copy| !copy.done)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn apply_blueprint(movement: &mut entries::EntryMove, mut recipe: CopyRecipe) {
    let options = movement.before.card.entry_copy_options();
    let added = movement.before.card.entry_copy_keywords();
    if added != Keywords::NONE || options.name.is_some() {
        let overrides = recipe.overrides.unwrap_or(CopyOverrides::NONE);
        recipe.overrides = Some(CopyOverrides {
            name: options.name.or(overrides.name),
            keywords: overrides.keywords.union(added),
            ..overrides
        });
    }
    if options.exceptions != CopyExceptions::NONE {
        recipe = recipe.with_exceptions(options.exceptions);
    }
    if movement.source_before.is_none() {
        movement.source_before = Some(Box::new(movement.before.clone()));
    }
    movement.before.card = recipe.definition;
    movement.before.name = recipe
        .overrides
        .and_then(|o| o.name)
        .unwrap_or_else(|| recipe.definition.name());
    movement.before.kinds = recipe.kinds();
    movement.before.colors = recipe.copiable_colors();
    movement.before.keywords = recipe
        .intrinsic_keywords()
        .union(recipe.overrides.map_or(Keywords::NONE, |o| o.keywords));
    movement.before.copy_recipe = Some(recipe);
    movement.exile_preparation = None;
    movement.devour_preparation = None;
    movement.choice = None;
    movement.life_paid = None;
    movement.attachment = None;
    movement.before.granted_subtypes = Game::entry_replacement_subtypes(movement);
    let base = recipe
        .copiable_base()
        .unwrap_or_else(|| match recipe.definition.effect() {
            Effect::Creature(p, t) => Stats {
                power: i128::from(p),
                toughness: i128::from(t),
            },
            _ => Stats::default(),
        });
    movement.before.power = base.power;
    movement.before.toughness = base.toughness;
    // A subsequent entry copy displaces the first copy's linked trigger (CR707.9g).
    movement.copy_preparation = Some(Box::new(Preparation {
        done: false,
        recipe: Some(recipe),
        exile: None,
    }));
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn candidate<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    movement: &entries::EntryMove,
    batch: &[entries::EntryMove],
    id: CardId,
) -> bool {
    let definition = movement.before.card;
    match definition.entry_copy_options().source {
        EntryCopySource::Battlefield => {
            !batch
                .iter()
                .any(|m| m.before.id == id && m.before.zone != Zone::Battlefield)
                && definition
                    .entry_copy()
                    .unwrap()
                    .filters
                    .iter()
                    .any(|&filter| {
                        battlefield_groups::filter_matches_in_world(
                            world,
                            filter,
                            movement.controller,
                            id,
                        )
                    })
        }
        EntryCopySource::GraveyardCreature { .. } => {
            // Simultaneous entrants are still in the graveyard during this choice.
            let object = world.object(id);
            object.zone == Zone::Graveyard
                && object.is_card()
                && object.card.front_face().is_creature()
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn chosen_blueprint<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    definition: Card,
    object: ObjectRef,
    battlefield_recipe: impl FnOnce() -> CopyRecipe,
) -> (CopyRecipe, Option<EntryCopyExile>) {
    match definition.entry_copy_options().source {
        EntryCopySource::Battlefield => (battlefield_recipe(), None),
        EntryCopySource::GraveyardCreature { exile_trigger } => (
            CopyRecipe::new(world.definition(object.card).front_face()),
            Some(EntryCopyExile {
                definition,
                ability: exile_trigger,
                object,
            }),
        ),
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn apply_choice(
    movement: &mut entries::EntryMove,
    (recipe, exile): (CopyRecipe, Option<EntryCopyExile>),
) {
    apply_blueprint(movement, recipe);
    movement.copy_preparation.as_mut().unwrap().exile = exile;
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
        (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| candidate(self, movement, &batch.moves, id))
            .map(|id| self.object_ref(id))
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
            let definition = self.state.entry_batch.as_ref().unwrap().moves[index]
                .before
                .card;
            let choice =
                chosen_blueprint(self, definition, object, || self.copiable_recipe(object));
            apply_choice(
                &mut self.state.entry_batch.as_mut().unwrap().moves[index],
                choice,
            );
        } else {
            self.state.entry_batch.as_mut().unwrap().moves[index]
                .copy_preparation
                .get_or_insert_default()
                .done = true;
        }
    }
    pub(super) fn queue_entry_copy_exile(
        &mut self,
        entrant: ObjectRef,
        controller: Player,
        exile: EntryCopyExile,
    ) {
        let mut source = self.object_view(entrant.card);
        debug_assert_eq!(source.reference, entrant);
        let facts = DamageFacts::from_view(&source);
        // The linked template belongs to the replacement that produced this
        // copy; the actual entrant supplies the reference and characteristics.
        source.card = exile.definition;
        source.controller = controller;
        // The replacement already created this ability, even if another
        // entrant has since caused the copied permanent to lose abilities.
        source.abilities = None;
        source.additional_triggers = 0;
        if self.queue_trigger(&source, usize::from(exile.ability)) {
            let packet = self.state.pending_triggers.last_mut().unwrap();
            packet.source.as_mut().unwrap().facts = facts;
            packet.set_event_object(EventObjectView {
                object: exile.object,
                facts: EventObjectFacts::Identity,
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
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
