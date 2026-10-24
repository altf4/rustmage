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
impl Game {
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
    pub(super) fn face_view_for(
        &self,
        object: ObjectRef,
        definition: Card,
        zone: Zone,
    ) -> Option<FaceView> {
        definition.other_face()?;
        Some(FaceView {
            physical_card: definition.front_face(),
            back: definition.face_parent().is_some(),
            transforms: if zone == Zone::Battlefield {
                self.transform_serial(object)
            } else {
                0
            },
        })
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
        let Some(other) = self.state.objects[item.card as usize].card.other_face() else {
            return;
        };
        if other.kind_mask() & (1 | 2 | 16 | 32 | 64 | 128) == 0 {
            return;
        }
        let serial = self
            .transform_serial(item.object)
            .checked_add(1)
            .capacity("transform count exhausted");
        self.forget_face_change(item.object);
        self.state
            .face_changes
            .get_or_insert_default()
            .push(FaceChange {
                object: item.object,
                serial,
            });
        self.remove_face_providers(item.card);
        self.state.objects[item.card as usize].card = other;
        if let Some(mut recipe) = self.copy_recipe(item.object) {
            recipe.definition = other;
            self.set_copy_recipe(item.object, recipe);
        }
        self.register_battlefield_providers(item.card);
        self.refresh_controllers();
    }
    fn remove_face_providers(&mut self, id: CardId) {
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
        for rows in [
            &mut self.state.static_sources,
            &mut self.state.cast_sources,
            &mut self.state.attack_sources,
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
