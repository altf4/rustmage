//! Modal faces share one physical object; transformation is never a zone transition.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FaceChange {
    object: ObjectRef,
    serial: u64,
}
pub(super) type Rows = Option<SharedVec<FaceChange>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<FaceChange>>() + r.len() * std::mem::size_of::<FaceChange>()
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FaceView {
    pub physical_card: Card,
    pub back: bool,
    pub transforms: u64,
}
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
impl Game {
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
        if !item.is_copy
            && item
                .program()
                .is_some_and(|(_, program)| program.contains(&Instruction::TransformSource))
        {
            item.source_transform = Some(self.transform_serial(item.object));
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
            Zone::Hand => o.owner == actor,
            Zone::Graveyard => self.has_graveyard_cast(actor, id, GraveyardCastKind::Cast),
            Zone::Exile => self.can_play_exiled(actor, id),
            Zone::Library => self.can_cast_library_top(actor, id),
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
    pub(super) fn transform_source(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card)
            || self.object_ref(item.card) != item.object
            || item.source_transform != Some(self.transform_serial(item.object))
        {
            return;
        }
        self.transform_permanent(item.object);
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
            return;
        }
        self.forget_hand_size_source(object);
        self.remove_face_providers(object.card);
        self.state.objects[object.card as usize].card = other;
        if let Some(mut recipe) = self.copy_recipe(object) {
            assert_eq!(recipe.turn_face(), Some(other));
            self.set_copy_recipe(object, recipe);
        }
        self.register_battlefield_providers(object.card);
        self.retimestamp_self_granted_statics(object);
        self.refresh_controllers();
    }
    pub(super) fn remove_face_providers(&mut self, id: CardId) {
        self.forget_trigger_multiplier_source(id);
        self.forget_ability_target_source(id);
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
            &mut self.state.counter_sources,
            &mut self.state.death_sources,
            &mut self.state.attachment_sources,
        ] {
            rows.retain(|source| *source != id);
        }
    }
}
