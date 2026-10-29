//! Copiable token exceptions, applied before later characteristic layers.
use super::*;

impl Game {
    pub(super) fn copiable_recipe(&self, object: ObjectRef) -> CopyRecipe {
        if self.is_face_down(object) {
            return CopyRecipe::new(self.state.objects[object.card as usize].card);
        }
        let recipe = self
            .copy_recipe(object)
            .unwrap_or_else(|| CopyRecipe::new(self.state.objects[object.card as usize].card));
        let faces = self.physical_copy_faces(object);
        if self
            .temporary_copy_row(object)
            .is_some_and(|row| !row.layers.is_empty())
        {
            recipe.with_copy_effect_faces(faces)
        } else {
            recipe.with_faces(faces)
        }
    }
    pub(super) fn recipe_for_subject(
        &self,
        item: &SpellView,
        subject: Subject,
    ) -> Option<CopyRecipe> {
        match subject {
            Subject::Source => {
                let source = item.source?;
                Some(if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object {
                    self.copiable_recipe(source.object)
                } else {
                    item.source_copy_recipe
                        .as_deref()
                        .copied()
                        .unwrap_or_else(|| CopyRecipe::new(source.card))
                })
            },
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
    pub(super) fn controls_planeswalker(&self, player: Player) -> bool {
        self.state.objects.iter().enumerate().any(|(id, object)| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && self.current_kinds(id as CardId) & 64 != 0
        })
    }
    pub(super) fn queue_copy_end_step_abilities(&mut self) {
        let sources: Vec<_> = self
            .state
            .copy_recipes
            .as_ref()
            .into_iter()
            .flat_map(|r| r.iter())
            .filter(|r| {
                self.on_battlefield(r.object.card)
                    && self.object_ref(r.object.card) == r.object
                    && self.printed_abilities_available(r.object.card)
            })
            .map(|r| {
                (
                    r.object.card,
                    r.recipe.exceptions.sacrifice_abilities,
                    r.recipe.exceptions.conditional_sacrifice_abilities,
                )
            })
            .collect();
        for (source, unconditional, conditional) in sources {
            let controller = self.state.objects[source as usize].controller;
            let conditional = if self.controls_planeswalker(controller) {
                0
            } else {
                conditional
            };
            for (amount, kind) in [
                (unconditional, StackKind::CopyEndStepSacrifice),
                (conditional, StackKind::CopyEndStepConditionalSacrifice),
            ] {
                for _ in 0..amount {
                    let mut item = self.activation_item(controller, source, 0);
                    item.kind = kind;
                    item.additional_triggers = trigger_multiplicity::additional_triggers(
                        &self.characteristic_query(),
                        source,
                    );
                    self.state.pending_triggers.push(item);
                }
            }
        }
    }
    pub(super) fn sacrifice_exact_source(&mut self, item: &SpellView) {
        self.try_sacrifice_exact_source(item);
    }
    pub(super) fn try_sacrifice_exact_source(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else { return false; };
        if !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
            || self.state.objects[source.id as usize].controller != item.controller
        {
            return false;
        }
        self.sacrifice_batch(&[(item.controller, source.id)]);
        // A replacement destination (for example exile) still completes the
        // sacrifice. A prevented movement must not create its reflexive payoff.
        self.object_ref(source.id) != source.object
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
        // Only the resulting exact graveyard incarnation qualifies. A previous
        // Food may have been a noncreature on the battlefield, so record cards,
        // not just creature deaths; resolution checks current graveyard types.
        if let Some(rows) = &mut self.state.battlefield_graveyard_entries
            && let Ok(at) = rows.binary_search_by_key(&before.id, |r| r.card)
        {
            rows.remove(at);
        }
        if destination == Zone::Graveyard
            && before.zone == Zone::Battlefield
            && !before.is_token()
            && !before.card.is_emblem()
        {
            let rows = self
                .state
                .battlefield_graveyard_entries
                .get_or_insert_default();
            let at = rows.partition_point(|r| r.card < after.card);
            rows.insert(at, after);
        }
        if self
            .state
            .battlefield_graveyard_entries
            .as_ref()
            .is_some_and(|r| r.is_empty())
        {
            self.state.battlefield_graveyard_entries = None;
        }
        if let Some(rows) = &mut self.state.graveyard_entries
            && let Ok(at) = rows.binary_search_by_key(&before.id, |r| r.card)
        {
            rows.remove(at);
        }
        if destination == Zone::Graveyard && self.state.objects[before.id as usize].is_card() {
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
