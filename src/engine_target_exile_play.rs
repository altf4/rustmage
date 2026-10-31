//! Exact-object target exile and independent owner/controller play permissions.
use super::*;
impl Game {
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
