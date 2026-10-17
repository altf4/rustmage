//! Fixed indefinite and sparse temporary control, and frozen non-targeted zone sets.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TemporaryControlView {
    pub object: ObjectRef,
    pub controller: Player,
    pub previous_controller: Player,
    pub expires_turn: u32,
}
impl Game {
    pub(super) fn remove_from_combat(&mut self, id: CardId, reference: ObjectRef) {
        self.state.attackers.retain(|a| a.card != id);
        self.state.blocking.retain(|&b| b != id);
        self.state.blocks.retain(|&(b, a)| b != id && a != id);
        self.state.selected_attackers.retain(|a| a.card != id);
        self.state
            .selected_blocks
            .retain(|&(b, a)| b != id && a != id);
        self.state.first_strikers.retain(|&c| c != id);
        for attack in self
            .state
            .attackers
            .iter_mut()
            .chain(&mut self.state.selected_attackers)
        {
            if attack.defender == Some(CombatDefender::Planeswalker(reference)) {
                attack.defender = None;
            }
        }
        // Surviving attackers remain blocked after their blockers leave combat.
    }
    fn change_controller(&mut self, object: ObjectRef, controller: Player) {
        if !self.on_battlefield(object.card)
            || self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].controller == controller
        {
            return;
        }
        self.remove_from_combat(object.card, object);
        let target = &mut self.state.objects[object.card as usize];
        target.controller = controller;
        target.summoning_sick = true;
    }
    pub(super) fn gain_control_until_end_turn(&mut self, item: &SpellView) {
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(id) = bound.target else {
                continue;
            };
            if !self.on_battlefield(id) {
                continue;
            }
            let effect = TemporaryControlView {
                object: self.object_ref(id),
                controller: item.controller,
                previous_controller: self.state.objects[id as usize].controller,
                expires_turn: self.state.turn,
            };
            self.state
                .temporary_controls
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(effect);
            self.change_controller(effect.object, effect.controller);
        }
    }
    pub(super) fn gain_control(
        &mut self,
        item: &SpellView,
        subject: Subject,
        controller: PlayerSet,
    ) {
        // Targeted players here are the already revalidated resolution targets.
        // Opponent uses the captured ability controller, not the live source's.
        let Some(controller) = self.program_players(controller, item).next() else {
            return;
        };
        for id in self.keyword_subjects(item, subject) {
            if !self.on_battlefield(id) {
                continue;
            }
            let object = self.object_ref(id);
            // This later fixed, indefinite effect supersedes every older control
            // effect for the recipient. Older EOT undo records must never restore
            // their controllers, even when the new controller is already current.
            // Future temporary effects capture this controller as their baseline.
            self.forget_temporary_control(object);
            self.change_controller(object, controller);
        }
    }
    pub(super) fn forget_temporary_control(&mut self, object: ObjectRef) {
        if let Some(effects) = &mut self.state.temporary_controls {
            effects.retain(|e| e.object != object);
        }
        if self
            .state
            .temporary_controls
            .as_ref()
            .is_some_and(|v| v.is_empty())
        {
            self.state.temporary_controls = None;
        }
    }
    pub(super) fn expire_temporary_controls(&mut self) {
        if let Some(effects) = self.state.temporary_controls.take() {
            // All current effects share this turn's cleanup. Reverse application
            // restores the controller before the earliest surviving exact-object effect.
            for effect in effects.iter().rev() {
                self.change_controller(effect.object, effect.previous_controller);
            }
        }
    }
    pub(super) fn temporary_control_bytes(&self) -> usize {
        self.state.temporary_controls.as_ref().map_or(0, |v| {
            std::mem::size_of::<Vec<TemporaryControlView>>()
                + v.len() * std::mem::size_of::<TemporaryControlView>()
        })
    }
    pub(super) fn return_graveyard_creatures(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
        maximum_mana_value: u64,
    ) {
        let mut owners = [false; 2];
        for p in self.program_players(players, item) {
            owners[p.index()] = true;
        }
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                if o.zone == Zone::Graveyard
                    && owners[o.owner.index()]
                    && o.card.is_creature()
                    && !o.card.is_token()
                    && o.card.fixed_mana_value() <= maximum_mana_value
                {
                    let mut movement = self.entry_move(id as CardId, Zone::Battlefield, false);
                    movement.controller = item.controller;
                    Some(movement)
                } else {
                    None
                }
            })
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn destroy_target_name_group(&mut self, item: &SpellView) {
        let Some(id) = item.targets.iter().find_map(|b| {
            if self.same_target_object(*b)
                && let Target::Card(id) = b.target
                && self.on_battlefield(id)
            {
                Some(id)
            } else {
                None
            }
        }) else {
            return;
        };
        // Token factories have stable synthetic identities; their actual names,
        // not those factory identifiers, determine shared-name destruction.
        let target_name = self.object_name(id);
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && (i as CardId == id
                        || !target_name.is_empty() && self.object_name(i as CardId) == target_name)
                    && !self
                        .keywords(i as CardId)
                        .contains(Keywords::INDESTRUCTIBLE))
                .then_some(i as CardId)
            })
            .collect();
        let sources = if ids.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.finish_zone_events(&events, &sources);
    }
}
