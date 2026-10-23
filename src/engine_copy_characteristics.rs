//! Copiable token exceptions, applied before later characteristic layers.
use super::*;

impl Game {
    pub(super) fn copiable_recipe(&self, object: ObjectRef) -> CopyRecipe {
        self.copy_recipe(object)
            .unwrap_or_else(|| CopyRecipe::new(self.state.objects[object.card as usize].card))
    }
    pub(super) fn copy_permanents_exact(&mut self, recipients: &[ObjectRef], recipe: CopyRecipe) {
        let mut recipients: Vec<_> = recipients
            .iter()
            .copied()
            .filter(|r| self.on_battlefield(r.card) && self.object_ref(r.card) == *r)
            .collect();
        recipients.sort_unstable_by_key(|r| r.card);
        recipients.dedup();
        if recipients.is_empty() {
            return;
        }
        for reference in &recipients {
            self.unregister_intrinsic_sources(reference.card);
        }
        let timestamp = self.next_role_timestamp();
        for reference in &recipients {
            let id = reference.card;
            let old = self.state.objects[id as usize].clone();
            let native = self
                .native_copy_recipe(*reference)
                .unwrap_or_else(|| self.copiable_recipe(*reference));
            let loyalty_used_turn = if old.used_abilities & old.card.loyalty_mask() != 0 {
                Some(self.state.turn)
            } else {
                self.state.copy_recipes.as_ref().and_then(|rows| {
                    rows.iter()
                        .find(|row| row.object == *reference)
                        .and_then(|row| row.loyalty_used_turn)
                })
            };
            self.clear_copy_recipe(*reference);
            let copied = recipe.overlay(native);
            let rows = self.state.copy_recipes.get_or_insert_with(Default::default);
            let at = rows.partition_point(|row| row.object.card < id);
            rows.insert(
                at,
                copies::CopyRow {
                    object: *reference,
                    recipe: copied,
                    native: Some(native),
                    loyalty_used_turn,
                    ability_origin: timestamp,
                },
            );
            let object = &mut self.state.objects[id as usize];
            object.card = copied.definition;
            object.copy_kind = if old.is_token() {
                CopyKind::Token
            } else {
                CopyKind::Physical
            };
            // A copy effect does not create an object. Once-per-planeswalker loyalty use persists.
            object.used_abilities = if loyalty_used_turn == Some(self.state.turn) {
                copied.definition.loyalty_mask()
            } else {
                0
            };
            self.clear_entry_value(*reference);
        }
        for reference in &recipients {
            self.register_battlefield_sources(reference.card);
        }
        // Gained static effects have the timestamp of this single copy effect, even across recipients.
        for reference in &recipients {
            self.set_static_source_timestamp(*reference, timestamp);
        }
        self.refresh_controllers();
    }
    pub(super) fn copied_keywords(&self, object: ObjectRef) -> Keywords {
        let mut result = Keywords::NONE;
        if let Some(recipe) = self.copy_recipe(object) {
            if recipe.exceptions.haste {
                result = result.union(Keywords::HASTE);
            }
            if recipe.exceptions.changeling {
                result = result.union(Keywords::CHANGELING);
            }
        }
        result
    }
    pub(super) fn recipe_for_subject(
        &self,
        item: &SpellView,
        subject: Subject,
    ) -> Option<CopyRecipe> {
        match subject {
            Subject::Source => Some(
                if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                    self.copiable_recipe(item.object)
                } else {
                    item.source_copy_recipe
                        .as_deref()
                        .map(|facts| facts.recipe)
                        .unwrap_or_else(|| CopyRecipe::new(item.source_card))
                },
            ),
            Subject::Targets => item.targets.iter().find_map(|bound| {
                if self.same_target_object(*bound)
                    && let Target::Card(id) = bound.target
                {
                    Some(self.copiable_recipe(self.object_ref(id)))
                } else {
                    None
                }
            }),
            _ => unreachable!("copy subject"),
        }
    }
    pub(super) fn queue_copy_end_step_abilities(&mut self) {
        let sources: Vec<_> = self
            .state
            .copy_recipes
            .as_ref()
            .into_iter()
            .flat_map(|r| r.iter())
            .filter(|r| {
                r.recipe.exceptions.sacrifice_abilities != 0
                    && self.on_battlefield(r.object.card)
                    && self.object_ref(r.object.card) == r.object
                    && self.printed_abilities_available(r.object.card)
            })
            .map(|r| (r.object.card, r.recipe.exceptions.sacrifice_abilities))
            .collect();
        for (source, amount) in sources {
            for _ in 0..amount {
                // Reuse the normal source-facts constructor; this is the token's
                // own intrinsic trigger, not a source-independent delayed effect.
                let mut item =
                    self.activation_item(self.state.objects[source as usize].controller, source, 0);
                item.kind = StackKind::CopyEndStepSacrifice;
                self.state.pending_triggers.push(item);
                let providers = self.freeze_trigger_providers();
                self.capture_intrinsic_trigger_multiplicity(
                    &self.object_view(source),
                    providers.as_deref(),
                    false,
                );
            }
        }
    }
    pub(super) fn sacrifice_exact_source(&mut self, item: &SpellView) {
        if self.on_battlefield(item.card)
            && self.object_ref(item.card) == item.object
            && self.state.objects[item.card as usize].controller == item.controller
        {
            self.sacrifice_batch(&[(item.controller, item.card)]);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct NightmareCull {
    candidates: Vec<ObjectRef>,
    protected: Vec<ObjectRef>,
}
pub(super) fn cull_bytes(cull: &Option<Box<NightmareCull>>) -> usize {
    cull.as_ref().map_or(0, |c| {
        std::mem::size_of::<NightmareCull>()
            + (c.candidates.len() + c.protected.len()) * std::mem::size_of::<ObjectRef>()
    })
}
impl Game {
    pub(super) fn graveyard_entry_this_turn(&self, object: ObjectRef) -> bool {
        self.state.graveyard_entries.as_ref().is_some_and(|rows| {
            rows.binary_search_by_key(&object.card, |r| r.card)
                .is_ok_and(|at| rows[at] == object)
        })
    }
    /// Called only after a real successful transition, with its final destination.
    pub(super) fn copy_characteristic_zone_commit(
        &mut self,
        before: &ObjectView,
        after: ObjectRef,
        destination: Zone,
    ) {
        if let Some(rows) = &mut self.state.graveyard_entries
            && let Ok(at) = rows.binary_search_by_key(&before.id, |r| r.card)
        {
            rows.remove(at);
        }
        if destination == Zone::Graveyard && !before.is_token() && !before.card.is_emblem() {
            let rows = self
                .state
                .graveyard_entries
                .get_or_insert_with(Default::default);
            let at = rows.partition_point(|r| r.card < after.card);
            rows.insert(at, after);
        }
        if self
            .state
            .graveyard_entries
            .as_ref()
            .is_some_and(|r| r.is_empty())
        {
            self.state.graveyard_entries = None;
        }
        if let Some(cull) = &mut self.state.nightmare_cull
            && let Some(at) = cull.candidates.iter().position(|r| *r == before.reference)
        {
            cull.candidates.swap_remove(at);
            if destination == Zone::Battlefield {
                cull.protected.push(after);
            }
        }
    }
    pub(super) fn track_nightmare_copy_entries(&mut self, moves: &[entries::EntryMove]) {
        if let Some(cull) = &mut self.state.nightmare_cull {
            cull.candidates
                .extend(moves.iter().map(|m| m.before.reference));
        }
    }
    pub(super) fn exile_and_copy_nightmare(&mut self, item: &SpellView) {
        let Some(id) = item.targets.iter().find_map(|b| match b.target {
            Target::Card(id) => Some(id),
            _ => None,
        }) else {
            return;
        };
        // Capture printed graveyard values before the attempted move. Failure or
        // replacement of that move does not cancel the following copy instruction.
        let mut recipe = CopyRecipe::new(self.state.objects[id as usize].card);
        recipe.exceptions = CopyExceptions::NIGHTMARE;
        self.move_object(id, Zone::Exile);
        self.state.nightmare_cull = Some(Default::default());
        self.stage_copy_creation(item.controller, recipe, 1);
    }
    pub(super) fn exile_other_nightmares(&mut self, player: Player) {
        let Some(cull) = self.state.nightmare_cull.take() else {
            return;
        };
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                let id = id as CardId;
                (o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.is_token()
                    && self.has_subtype(id, "Nightmare")
                    && !cull.protected.contains(&self.object_ref(id)))
                .then_some(id)
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.finish_zone_events(&events, &[]);
    }
}
