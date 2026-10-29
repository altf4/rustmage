//! Fixed indefinite and sparse temporary control, and frozen non-targeted zone sets.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TemporaryControlView {
    pub object: ObjectRef,
    pub controller: Player,
    pub previous_controller: Player,
    pub expires_turn: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after_next_turn: Option<NextTurnControlExpiry>,
}
/// A duration tied to an actual future turn, including extra or skipped turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NextTurnControlExpiry {
    pub player: Player,
    pub created_turn: u32,
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
            if matches!(attack.defender, Some(CombatDefender::Planeswalker(r) | CombatDefender::Battle(r)) if r == reference)
            {
                attack.defender = None;
            }
        }
        // Surviving attackers remain blocked after their blockers leave combat.
    }
    pub(super) fn change_controller(&mut self, object: ObjectRef, controller: Player) {
        if !self.on_battlefield(object.card)
            || self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].controller == controller
        {
            return;
        }
        let previous = self.state.objects[object.card as usize].controller;
        self.remember_face_down_controller(object, previous);
        self.remember_face_down_controller(object, controller);
        self.end_control_linked_effects(object);
        self.remember_hideaway_controller(object, controller);
        self.remove_from_combat(object.card, object);
        let target = &mut self.state.objects[object.card as usize];
        target.controller = controller;
        target.summoning_sick = true;
    }
    pub(super) fn gain_control_until_end_turn(&mut self, item: &SpellView) {
        self.gain_control_with_duration(item, false);
    }
    pub(super) fn gain_control_conditional_villain_mana_value(&mut self, item: &SpellView) {
        self.gain_control_with_duration(item, true);
    }
    fn gain_control_with_duration(&mut self, item: &SpellView, conditional: bool) {
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
            let longer = conditional
                && self.state.objects.iter().enumerate().any(|(other, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == item.controller
                        && self.has_subtype(other as CardId, "Villain")
                        && self.object_mana_value(other as CardId) > self.object_mana_value(id)
                });
            let effect = TemporaryControlView {
                object: self.object_ref(id),
                controller: item.controller,
                previous_controller: self.state.objects[id as usize].controller,
                expires_turn: (!longer).then_some(self.state.turn),
                expires_after_next_turn: longer.then_some(NextTurnControlExpiry {
                    player: item.controller,
                    created_turn: self.state.turn,
                }),
            };
            self.state
                .temporary_controls
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(effect);
            self.record_control_expiring(
                effect.object,
                effect.controller,
                effect.expires_turn,
                effect.expires_after_next_turn,
            );
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
            if let Some(state) = &mut self.state.controls {
                state.effects.retain(|row| {
                    row.object != object
                        || (row.expires_turn.is_none() && row.expires_after_next_turn.is_none())
                });
            }
            self.record_control(object, controller, None);
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
        let turn = self.state.turn;
        let active = self.state.active;
        let expired = |eot: Option<u32>, next: Option<NextTurnControlExpiry>| {
            eot.is_some_and(|end| end <= turn)
                || next.is_some_and(|end| end.player == active && turn > end.created_turn)
        };
        if let Some(rows) = &mut self.state.temporary_controls {
            rows.retain(|row| !expired(row.expires_turn, row.expires_after_next_turn));
            if rows.is_empty() {
                self.state.temporary_controls = None;
            }
        }
        if let Some(state) = &mut self.state.controls {
            state
                .effects
                .retain(|row| !expired(row.expires_turn, row.expires_after_next_turn));
        }
        self.refresh_controllers();
    }
    pub(super) fn temporary_control_bytes(&self) -> usize {
        self.state.temporary_controls.as_ref().map_or(0, |v| {
            std::mem::size_of::<Vec<TemporaryControlView>>()
                + v.len() * std::mem::size_of::<TemporaryControlView>()
        })
    }
    pub(super) fn return_graveyard_mana_value(
        &mut self,
        item: &SpellView,
        kinds: u16,
        value: AmountValue,
    ) {
        let value = self.item_amount(value, item);
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Graveyard
                    && o.owner == item.controller
                    && o.is_card()
                    && o.card.kind_mask() & kinds != 0
                    && u128::from(o.card.fixed_mana_value()) == value
            })
            .map(|(id, _)| self.entry_move(id as CardId, Zone::Battlefield, false))
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn return_owned_graveyard_nonland_permanents(&mut self, item: &SpellView) {
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let kinds = object.card.kind_mask();
                (object.zone == Zone::Graveyard
                    && object.owner == item.controller
                    && object.is_card()
                    && kinds & (1 | 16 | 32 | 64 | 256) != 0
                    && kinds & 2 == 0)
                    .then(|| {
                        let mut movement = self.entry_move(id as CardId, Zone::Battlefield, false);
                        movement.controller = item.controller;
                        movement
                    })
            })
            .collect();
        self.stage_entries(moves, None);
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
                    && o.is_card()
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
        let mut ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && (i as CardId == id
                        || crate::card_names::same_name(
                            self.object_name(i as CardId),
                            target_name,
                        ))
                    && !self
                        .keywords(i as CardId)
                        .contains(Keywords::INDESTRUCTIBLE))
                .then_some(i as CardId)
            })
            .collect();
        ids.retain(|&id| !self.effect_destruction_prevented(id));
        let sources = if ids.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.finish_zone_events(&events, &sources);
    }
}

impl Game {
    pub(super) fn exchange_control_targets(&mut self, item: &SpellView) {
        let targets: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                if self.same_target_object(*bound) && self.on_battlefield(id) {
                    Some(self.object_ref(id))
                } else {
                    None
                }
            })
            .collect();
        if let [first, second] = targets.as_slice() {
            self.record_control_exchange(*first, *second);
        }
    }
}
