//! Exact linked returns, turn entry membership and accepted private wheel shuffles.
use super::*;
impl Game {
    fn farm_linked_objects(&self, item: &SpellView) -> Vec<ObjectRef> {
        self.state
            .linked_choices
            .as_ref()
            .map_or_else(Vec::new, |data| {
                data.links
                    .iter()
                    .filter(|link| {
                        link.source == item.object
                            && link.ability_origin == item.ability_origin()
                            && self.object_ref(link.exiled.card) == link.exiled
                            && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                            && !self.state.objects[link.exiled.card as usize].is_token()
                    })
                    .map(|link| link.exiled)
                    .collect()
            })
    }
    pub(super) fn linked_creature_return_candidate(&self, object: ObjectRef) -> bool {
        let item = &self.state.resolving.as_ref().unwrap().item;
        let card = &self.state.objects[object.card as usize];
        card.zone == Zone::Exile
            && card.card.is_creature()
            && !card.is_token()
            && self.object_ref(object.card) == object
            && self.state.linked_choices.as_ref().is_some_and(|data| {
                data.links.iter().any(|link| {
                    link.source == item.object
                        && link.ability_origin == item.ability_origin()
                        && link.exiled == object
                })
            })
    }
    pub(super) fn complete_linked_creature_return(
        &mut self,
        choice: &effect_choices::EffectChoice,
    ) {
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        let selected: Vec<_> = choice.selected[player.index()]
            .iter()
            .copied()
            .filter(|&object| self.linked_creature_return_candidate(object))
            .collect();
        // The chosen card is excluded from "each other" even if entry is replaced.
        self.state.resolving.as_mut().unwrap().milled = selected.clone();
        let moves = selected
            .iter()
            .map(|object| {
                let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
                movement.controller = player;
                movement
            })
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn return_other_linked_exile(&mut self, item: &SpellView) {
        let chosen = &self.state.resolving.as_ref().unwrap().milled;
        let moves = self
            .farm_linked_objects(item)
            .into_iter()
            .filter(|object| !chosen.contains(object))
            .map(|object| self.entry_move(object.card, Zone::Hand, false))
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn return_new_nonland_permanents(&mut self, item: &SpellView) {
        let ids: Vec<_> = self
            .state
            .permanent_entry_history
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|&&object| {
                object != item.object
                    && self.on_battlefield(object.card)
                    && self.object_ref(object.card) == object
                    && self.current_kinds(object.card) & 2 == 0
            })
            .map(|object| object.card)
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Hand);
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn shuffle_wheel_hands_and_graveyards(&mut self, item: &SpellView) {
        let players: Vec<_> = self.program_players(PlayerSet::Wheelers, item).collect();
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                players.contains(&object.owner)
                    && matches!(object.zone, Zone::Hand | Zone::Graveyard)
            })
            .map(|(id, _)| id as CardId)
            .collect();
        // This is neither discard nor targeted movement. No private hand identity
        // is published in TargetLibraryMovement or remembered as a library top.
        let events = self.move_objects_raw(&ids, Zone::Library);
        self.finish_zone_events(&events, &[]);
        for player in players {
            self.forget_library_top(player);
            self.state
                .rng
                .shuffle(&mut self.state.players[player.index()].library);
        }
    }
}
