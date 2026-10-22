//! Face changes preserve the permanent's incarnation and all external effects.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TransformHistoryView {
    pub object: ObjectRef,
    pub timestamp: u64,
}
impl Game {
    fn transform_timestamp(&self, object: ObjectRef) -> u64 {
        self.state
            .roles
            .as_ref()
            .and_then(|roles| roles.transforms.iter().find(|row| row.object == object))
            .map_or(0, |row| row.timestamp)
    }
    pub(super) fn capture_transform_stamp(&self, item: &mut SpellView) {
        if item.transform_stamp.is_none()
            && item
                .program()
                .is_some_and(|(_, program)| program.contains(&Instruction::TransformSource))
        {
            item.transform_stamp = Some(Box::new(self.transform_timestamp(item.object)));
        }
    }
    pub(super) fn forget_transform_history(&mut self, object: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles.transforms.retain(|row| row.object != object);
        }
    }
    pub(super) fn transform_source(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card)
            || self.object_ref(item.card) != item.object
            || item
                .transform_stamp
                .as_deref()
                .is_some_and(|&stamp| stamp != self.transform_timestamp(item.object))
        {
            return;
        }
        self.transform_permanent(item.card);
    }
    pub(super) fn transform_permanent(&mut self, id: CardId) {
        let old = self.state.objects[id as usize].card;
        let Some(spec) = old.transform_spec() else {
            return;
        };
        let next = if old == spec.front {
            spec.back
        } else {
            spec.front
        };
        let object = self.object_ref(id);
        // Remove only intrinsic providers. Resolution-created effects and combat membership survive.
        if let Some(roles) = &mut self.state.roles {
            roles
                .base_stats
                .retain(|row| !row.static_source || row.source != object);
            roles.live_static.retain(|row| row.source != object);
            roles.ability_timestamps.retain(|row| row.source != object);
            roles
                .ability_removal_sources
                .retain(|&source| source != object);
            roles
                .characteristic_sources
                .retain(|&source| source != object);
        }
        for sources in [
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
            sources.retain(|&source| source != id);
        }
        let used_loyalty = self.state.objects[id as usize].used_abilities & old.loyalty_mask() != 0;
        self.state.objects[id as usize].card = next;
        if used_loyalty {
            self.state.objects[id as usize].used_abilities |= next.loyalty_mask();
        }
        if let Some(rows) = &mut self.state.copy_recipes {
            for row in rows.iter_mut().filter(|row| row.object == object) {
                row.recipe.definition = next;
            }
        }
        let timestamp = self.next_role_timestamp();
        self.forget_transform_history(object);
        self.state
            .roles
            .as_mut()
            .unwrap()
            .transforms
            .push(TransformHistoryView { object, timestamp });
        self.register_battlefield_sources(id);
        self.assign_static_source_timestamp(object);
        self.refresh_controllers();
        let source = self.object_view(id);
        for (index, ability) in next.triggers().iter().enumerate() {
            if matches!(
                ability.condition.event(),
                TriggerCondition::SelfTransforms | TriggerCondition::SelfEntersOrTransforms
            ) {
                self.queue_trigger(&source, index);
            }
        }
    }
}
