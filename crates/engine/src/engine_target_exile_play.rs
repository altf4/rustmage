//! Exact-object target exile and independent owner/controller play permissions.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn airbend_targets(&mut self, item: &SpellView) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .copied()
            .filter(|&t| self.same_target_object(t))
            .filter_map(|t| match t.target {
                Target::Card(id) | Target::Spell(id) => Some(id),
                Target::Stack(_) => self
                    .bound_stack_item(&t)
                    .filter(|victim| victim.kind == StackKind::Spell)
                    .and_then(|victim| victim.source.map(|source| source.id)),
                _ => None,
            })
            .collect();
        self.airbend_objects(&ids);
    }
    pub(super) fn airbend_objects(&mut self, ids: &[CardId]) {
        let events = self.move_objects_raw(ids, Zone::Exile);
        for event in &events {
            if event.moved
                && event.destination == Zone::Exile
                && !event.before.is_token()
                && !self.state.objects[event.after.card as usize].is_spell_copy()
            {
                self.grant_airbend_permission(event.before.owner, event.after);
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    /// Airbend the current creature set as one simultaneous zone-change batch.
    /// The exception is an exact incarnation, never a reused card ID.
    pub(super) fn airbend_other_creatures(&mut self, exception: Option<ObjectRef>) {
        let ids: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                self.state.objects[id as usize].zone == Zone::Battlefield
                    && self.current_kinds(id) & 1 != 0
                    && exception != Some(self.object_ref(id))
            })
            .collect();
        self.airbend_objects(&ids);
    }

    pub(super) fn exile_targets_play(
        &mut self,
        item: &SpellView,
        next_turn: bool,
        owners: bool,
        include_top: bool,
    ) {
        let mut ids: Vec<_> = item
            .targets
            .iter()
            .filter(|&&t| self.same_target_object(t))
            .filter_map(|t| {
                if let Target::Card(id) = t.target {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        if include_top && let Some(top) = self.top_library_card(item.controller) {
            ids.push(top);
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let moved = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Exile)
            .count();
        self.state.resolving.as_mut().unwrap().moved_count =
            Some(u64::try_from(moved).capacity("exiled card count exhausted"));
        for event in &events {
            if event.moved
                && event.destination == Zone::Exile
                && !event.before.is_token()
                && !self.state.objects[event.after.card as usize].is_spell_copy()
            {
                self.grant_exile_play_permission(
                    if owners {
                        event.before.owner
                    } else {
                        item.controller
                    },
                    event.after,
                    next_turn,
                );
            }
        }
        self.finish_zone_events(&events, &[]);
    }
}
