//! Sparse live source clauses use current earlier-layer facts and a shared timestamp clock.
use super::*;
use crate::cards::LiveStaticEffect;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LiveStaticSourceView {
    pub source: ObjectRef,
    pub timestamp: u64,
    /// Immutable catalogue data; independent of every game/parent allocation.
    pub effects: &'static [LiveStaticEffect],
}

impl Game {
    pub(super) fn register_live_static_source(&mut self, id: CardId) {
        let effects = self.state.objects[id as usize].card.live_static_effects();
        self.register_live_static_effects(id, effects);
    }
    fn register_live_static_effects(&mut self, id: CardId, effects: &'static [LiveStaticEffect]) {
        if effects.is_empty() {
            return;
        }
        let source = self.object_ref(id);
        if self
            .state
            .roles
            .as_ref()
            .is_some_and(|r| r.live_static.iter().any(|e| e.source == source))
        {
            return;
        }
        let timestamp = self.static_source_timestamp(source);
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .live_static
            .push(LiveStaticSourceView {
                source,
                timestamp,
                effects,
            });
    }
    pub(super) fn forget_live_static_source(&mut self, source: ObjectRef) {
        if let Some(r) = &mut self.state.roles {
            r.live_static.retain(|e| e.source != source);
        }
    }
    pub(super) fn retimestamp_live_static_source(&mut self, source: ObjectRef) {
        if self.temporary_copy_row(source).is_some() {
            self.assign_static_source_timestamp(source);
            return;
        }
        if !self.state.roles.as_ref().is_some_and(|r| {
            r.live_static.iter().any(|e| e.source == source)
                || r.ability_timestamps.iter().any(|e| e.source == source)
        }) {
            return;
        }
        self.assign_static_source_timestamp(source);
    }

    pub(super) fn live_static_forbids_block(&self, blocker: CardId, attacker: CardId) -> bool {
        self.characteristic_query()
            .live_static_forbids_block(blocker, attacker)
    }
}
