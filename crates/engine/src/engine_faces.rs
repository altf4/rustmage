//! In-place face changes and exact physical-card entry effects.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FaceChange {
    object: ObjectRef,
    serial: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<FaceChange>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<FaceChange>>() + r.len() * std::mem::size_of::<FaceChange>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FaceView {
    pub physical_card: Card,
    pub back: bool,
    pub transforms: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FaceView {
    pub(super) fn from_faces(faces: CopyFaces, transforms: u64) -> Option<Self> {
        let CopyFaces::Double {
            physical_front,
            back_up,
            ..
        } = faces
        else {
            return None;
        };
        Some(Self {
            physical_card: physical_front,
            back: back_up,
            transforms,
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A natural transformed back uses its front's mana cost; copying that
    /// back face instead yields zero, even on another double-faced permanent.
    pub(super) fn object_mana_value(&self, id: CardId) -> u64 {
        ability_world::CharacteristicWorld::mana_value(self, id)
    }
    pub(super) fn physical_copy_faces(&self, object: ObjectRef) -> CopyFaces {
        let recipe = if let Some(row) = self.temporary_copy_row(object) {
            row.original_recipe
                .unwrap_or_else(|| CopyRecipe::new(self.physical_definition(object.card)))
        } else {
            self.copy_recipe(object)
                .unwrap_or_else(|| CopyRecipe::new(self.state.objects[object.card as usize].card))
        };
        recipe.face_structure()
    }
    pub(super) fn capture_transform_stack_serial(&self, item: &mut SpellView) {
        let tracks_transform = if item.is_copy {
            // A copied ability inherits the tracked source, but gets a fresh
            // timestamp when this packet is put on the stack (CR 701.27f).
            item.kind != StackKind::Spell && item.source_transform.is_some()
        } else {
            item.program()
                .is_some_and(|(_, program)| program.contains(&Instruction::TransformSource))
        };
        if tracks_transform {
            item.source_transform = item
                .source
                .map(|source| self.transform_serial(source.object));
        }
    }
    pub(super) fn transform_serial(&self, object: ObjectRef) -> u64 {
        self.state
            .face_changes
            .as_ref()
            .and_then(|r| r.iter().find(|r| r.object == object))
            .map_or(0, |r| r.serial)
    }
    pub(super) fn forget_face_change(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.face_changes {
            rows.retain(|r| r.object != object);
            if rows.is_empty() {
                self.state.face_changes = None;
            }
        }
    }
    pub(super) fn back_cast_available(&self, actor: Player, id: CardId, sorcery: bool) -> bool {
        let o = &self.state.objects[id as usize];
        let allowed = match o.zone {
            Zone::Hand => o.owner == actor && o.is_card(),
            Zone::Graveyard => self.has_graveyard_cast(actor, id, GraveyardCastKind::Cast),
            Zone::Exile => self.can_play_exiled(actor, id),
            Zone::Library => o
                .card
                .back_face()
                .is_some_and(|definition| self.can_cast_library_face(actor, id, definition)),
            _ => false,
        };
        allowed
            && self.permitted_cast_available(
                actor,
                id,
                CastMethod::Normal,
                CastFace::Back,
                sorcery,
                false,
            )
    }
    /// Exile the resolving physical card and apply its added entry counter.
    pub(super) fn exile_source_return_transformed(
        &mut self,
        item: &SpellView,
        counter: CounterKind,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if item.is_copy || source.card.back_face().is_none() {
            return;
        }
        if let Some(mut entry) =
            self.exile_for_source_return(source.object, Zone::Stack, true, None)
        {
            entry.effect_counter = Some((counter, 1));
            self.stage_entries(vec![entry], None);
        }
    }

    pub(super) fn exile_battlefield_source_return(
        &mut self,
        item: &SpellView,
        transformed: bool,
        under_owner: bool,
    ) {
        if let Some(source) = item.source
            && let Some(event) = self.exile_moving_source(source.object, Zone::Battlefield)
        {
            let entries = event
                .objects_after()
                .filter_map(|object| {
                    self.source_return_entry(
                        object,
                        Zone::Exile,
                        transformed,
                        (!under_owner).then_some(item.controller),
                    )
                })
                .collect();
            self.stage_entries(entries, None);
        }
    }

    pub(super) fn return_graveyard_source_transformed(&mut self, item: &SpellView) {
        if let Some(source) = item.source
            && let Some(entry) = self.source_return_entry(
                source.object,
                Zone::Graveyard,
                true,
                Some(item.controller),
            )
        {
            self.stage_entries(vec![entry], None);
        }
    }

    fn exile_for_source_return(
        &mut self,
        object: ObjectRef,
        origin: Zone,
        transformed: bool,
        controller: Option<Player>,
    ) -> Option<entries::EntryMove> {
        let event = self.exile_moving_source(object, origin)?;
        self.source_return_entry(event.after, Zone::Exile, transformed, controller)
    }

    fn exile_moving_source(&mut self, object: ObjectRef, origin: Zone) -> Option<ZoneChange> {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != origin
        {
            return None;
        }
        let event = self.move_object(object.card, Zone::Exile);
        (event.moved && event.destination == Zone::Exile).then_some(event)
    }

    /// Craft pays by moving the source to exile before the ability resolves.
    /// Only the exile incarnation recorded by that payment can return.
    pub(super) fn return_crafted_source_transformed(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let Some(movement) = item.source_move.as_ref().filter(|movement| {
            movement.before == source.object
                && movement.origin == Zone::Battlefield
                && movement.destination == Zone::Exile
        }) else {
            return;
        };
        if let Some(mut entry) = self.source_return_entry(movement.after, Zone::Exile, true, None) {
            entry.craft_materials = item.craft_materials.clone();
            self.stage_entries(vec![entry], None);
        }
    }

    /// The zone change has restored printed identity. Project the entering face
    /// for replacements and choices while retaining the actual pre-entry view.
    fn source_return_entry(
        &self,
        object: ObjectRef,
        origin: Zone,
        transformed: bool,
        controller: Option<Player>,
    ) -> Option<entries::EntryMove> {
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object || card.zone != origin || !card.is_card() {
            return None;
        }
        let front = card.card.front_face();
        let face = if transformed {
            front.back_face()?
        } else {
            front
        };
        if face.kind_mask() & (1 | 2 | 16 | 32 | 64 | crate::cards::BATTLE_KIND) == 0 {
            return None;
        }

        let mut entry = self.entry_move(object.card, Zone::Battlefield, false);
        entry.controller = controller.unwrap_or(card.owner);
        if transformed {
            entry.project_printed_face(face);
        }
        Some(entry)
    }

    pub(super) fn transform_source(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
            || item.source_transform != Some(self.transform_serial(source.object))
        {
            return;
        }
        self.transform_permanent(source.object);
    }
    pub(super) fn can_transform_permanent(&self, object: ObjectRef) -> bool {
        self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && self
                .physical_copy_faces(object)
                .turned()
                .is_some_and(|(_, other)| other.kind_mask() & (1 | 2 | 16 | 32 | 64 | 128) != 0)
    }
    pub(super) fn transform_permanent(&mut self, object: ObjectRef) {
        if !self.can_transform_permanent(object) {
            return;
        }
        let Some((_, other)) = self.physical_copy_faces(object).turned() else {
            return;
        };
        if other.kind_mask() & (1 | 2 | 16 | 32 | 64 | 128) == 0 {
            return;
        }
        let serial = self
            .transform_serial(object)
            .checked_add(1)
            .capacity("transform count exhausted");
        self.forget_face_change(object);
        self.state
            .face_changes
            .get_or_insert_default()
            .push(FaceChange { object, serial });
        if self.transform_under_temporary_copy(object, other) {
            self.queue_self_transform_triggers(object);
            return;
        }
        let control_watch = self.freeze_control_watch();
        self.forget_hand_size_source(object);
        self.remove_face_providers(object.card);
        self.state.objects[object.card as usize].card = other;
        if let Some(mut recipe) = self.copy_recipe(object) {
            assert_eq!(recipe.turn_face(), Some(other));
            self.set_copy_recipe(object, recipe);
        }
        self.register_battlefield_providers(object.card);
        self.retimestamp_self_granted_statics(object);
        self.refresh_controllers_after(Some(control_watch));
        self.queue_self_transform_triggers(object);
    }
    fn queue_self_transform_triggers(&mut self, object: ObjectRef) {
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if matches!(
                ability.condition.event(),
                TriggerCondition::SelfTransforms | TriggerCondition::SelfEntersOrTransforms
            ) {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn remove_face_providers(&mut self, id: CardId) {
        self.forget_trigger_multiplier_source(id);
        self.forget_activation_observer(id);
        self.forget_ability_target_source(id);
        self.forget_control_change_source(id);
        let reference = self.object_ref(id);
        // Keep recipient effects and counters: only the old face's intrinsic providers change.
        if let Some(roles) = &mut self.state.roles {
            roles
                .base_stats
                .retain(|r| !(r.static_source && r.source == reference));
            roles.live_static.retain(|r| r.source != reference);
            roles.ability_timestamps.retain(|r| r.source != reference);
            roles.ability_removal_sources.retain(|r| *r != reference);
            roles.characteristic_sources.retain(|r| *r != reference);
        }
        if let Some(data) = &mut self.state.explore_data {
            data.sources.retain(|&source| source != id);
        }
        if let Some(data) = &mut self.state.evidence_data {
            data.sources.retain(|&source| source != id);
        }
        for rows in [
            &mut self.state.static_sources,
            &mut self.state.cast_sources,
            &mut self.state.attack_sources,
            &mut self.state.tap_sources,
            &mut self.state.damage_sources,
            &mut self.state.discard_sources,
            &mut self.state.draw_sources,
            &mut self.state.step_sources,
            &mut self.state.life_sources,
            &mut self.state.scry_sources,
            &mut self.state.counter_sources,
            &mut self.state.death_sources,
            &mut self.state.attachment_sources,
        ] {
            rows.retain(|source| *source != id);
        }
    }
}
