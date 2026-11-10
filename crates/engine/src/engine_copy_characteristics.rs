//! Copiable token exceptions, applied before later characteristic layers.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
                Some(
                    if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object
                    {
                        self.copiable_recipe(source.object)
                    } else {
                        item.source_copy_recipe
                            .as_deref()
                            .copied()
                            .unwrap_or_else(|| CopyRecipe::new(source.card))
                    },
                )
            }
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
                    r.recipe
                        .exceptions
                        .sacrifice_abilities
                        .checked_add(r.recipe.exceptions.sacrifice_only_abilities)
                        .capacity("copy sacrifice abilities"),
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
    fn exact_controlled_sacrifice_source(&self, item: &SpellView) -> Option<StackSourceView> {
        item.source.filter(|source| {
            self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object
                && self.state.objects[source.id as usize].controller == item.controller
        })
    }
    pub(super) fn exact_source_can_sacrifice(&self, item: &SpellView) -> bool {
        self.exact_controlled_sacrifice_source(item)
            .is_some_and(|source| {
                !ability_world::CharacteristicWorld::sacrifice_prohibited(self, source.id)
            })
    }
    pub(super) fn try_sacrifice_exact_source(&mut self, item: &SpellView) -> bool {
        let Some(source) = self.exact_controlled_sacrifice_source(item) else {
            return false;
        };
        self.sacrifice_batch(&[(item.controller, source.id)]);
        // A replacement destination (for example exile) still completes the
        // sacrifice. A prevented movement must not create its reflexive payoff.
        self.object_ref(source.id) != source.object
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct NightmareCull {
    candidates: Vec<ObjectRef>,
    protected: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn cull_bytes(cull: &Option<Box<NightmareCull>>) -> usize {
    cull.as_ref().map_or(0, |c| {
        std::mem::size_of::<NightmareCull>()
            + (c.candidates.len() + c.protected.len()) * std::mem::size_of::<ObjectRef>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
        self.forget_mayhem_discard(before.reference);
        if before.zone == Zone::Battlefield {
            self.forget_block_restriction(before.reference);
        }
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
    pub(super) fn exile_and_copy_creature_as(
        &mut self,
        item: &SpellView,
        base: (i16, i16),
        color: crate::cards::ChoiceColor,
        subtype: &'static str,
    ) {
        let Some(id) = item.targets.iter().find_map(|bound| {
            let Target::Card(id) = bound.target else {
                return None;
            };
            let object = self.state.objects.get(id as usize)?;
            (self.same_target_object(*bound)
                && object.zone == Zone::Graveyard
                && object.is_card()
                && object.card.is_creature())
            .then_some(id)
        }) else {
            return;
        };
        let definition = self.state.objects[id as usize].card;
        let event = self.move_object(id, Zone::Exile);
        if !event.moved || event.destination != Zone::Exile || self.object_ref(id) != event.after {
            return;
        }
        let mut recipe = CopyRecipe::new(definition).with_exceptions(CopyExceptions {
            base: Some(base),
            ..CopyExceptions::NONE
        });
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1),
            types: Some(CopyTypes {
                replace: true,
                ..CopyTypes::default()
            }),
            ..CopyOverrides::NONE
        });
        recipe.color_override = Some(1 << color.index());
        recipe.replacement_subtype = Some(
            crate::cards::CreatureType::from_name(subtype).expect("valid copy creature subtype"),
        );
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
