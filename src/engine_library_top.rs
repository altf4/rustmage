//! Private CR401.5 operation boundaries and public, historical one-card reveals.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryRevealEvent {
    pub sequence: u64,
    pub source: ObjectRef,
    pub player: Player,
    pub revealed: RevealedLibraryCard,
    /// Historical result, not an assertion about the card's current zone.
    pub drawn: Option<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VisibilityOperation {
    pub starting_top: [Option<ObjectRef>; 2],
    pub changed: [bool; 2],
    pub land: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LibraryTopState {
    pub operation: Option<VisibilityOperation>,
    pub reveal: Option<LibraryRevealEvent>,
}
impl Game {
    pub(super) fn prepare_library_visibility(&mut self, action: &Action) {
        if self
            .state
            .library_top
            .as_ref()
            .is_some_and(|s| s.operation.is_some())
        {
            return;
        }
        if !self.state.objects.iter().any(|o| {
            o.card.static_ability_groups().iter().any(|g| {
                g.fragments
                    .iter()
                    .any(|f| matches!(f, crate::cards::StaticAbilityFragment::LookAtLibraryTop))
            })
        }) {
            return;
        }
        let starting_top = std::array::from_fn(|i| {
            self.state.players[i]
                .library
                .last()
                .map(|&id| self.object_ref(id))
        });
        self.state.library_top.get_or_insert_default().operation = Some(VisibilityOperation {
            starting_top,
            changed: [false; 2],
            land: matches!(
                action,
                Action::PlayLand { .. }
                    | Action::PlayGraveyardPermanent {
                        permanent_type: PermanentType::Land,
                        ..
                    }
            ),
        });
    }
    pub(super) fn finish_library_visibility(&mut self) {
        let ongoing = self.state.announcing.as_ref().is_some_and(|a| !a.trigger)
            || self.state.mana_announcement_stack.is_some()
            || self.plot_payment().is_some()
            || (self.state.entry_batch.is_some()
                && self
                    .state
                    .library_top
                    .as_ref()
                    .and_then(|s| s.operation.as_ref())
                    .is_some_and(|o| o.land));
        if !ongoing && let Some(s) = &mut self.state.library_top {
            s.operation = None;
            if s.reveal.is_none() {
                self.state.library_top = None;
            }
        }
    }
    pub(super) fn library_top_changed(&mut self, player: Player) {
        if let Some(o) = self
            .state
            .library_top
            .as_mut()
            .and_then(|s| s.operation.as_mut())
        {
            o.changed[player.index()] = true;
        }
    }
    pub(super) fn library_top_deferred(&self, player: Player) -> bool {
        self.state
            .library_top
            .as_ref()
            .and_then(|s| s.operation.as_ref())
            .is_some_and(|o| {
                o.changed[player.index()]
                    || o.starting_top[player.index()]
                        != self.state.players[player.index()]
                            .library
                            .last()
                            .map(|&id| self.object_ref(id))
            })
    }
    pub(super) fn reveal_top_move_if_permanent(&mut self, item: &SpellView) {
        let player = item.controller;
        let Some(&id) = self.state.players[player.index()].library.last() else {
            return;
        };
        let revealed = RevealedLibraryCard {
            object: self.object_ref(id),
            card: self.state.objects[id as usize].card,
        };
        self.remember_library_top(player, revealed.object);
        let sequence = self
            .state
            .library_top
            .as_ref()
            .and_then(|s| s.reveal.as_ref())
            .map_or(0, |e| e.sequence)
            .checked_add(1)
            .capacity("library reveal sequence exhausted");
        self.state.library_top.get_or_insert_default().reveal = Some(LibraryRevealEvent {
            sequence,
            source: item.source.expect("library effect requires source").object,
            player,
            revealed,
            drawn: None,
        });
        if revealed.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 128 | 256) != 0 {
            let event = self.move_object(id, Zone::Hand);
            if event.moved
                && event.destination == Zone::Hand
                && !self.state.known_hand.contains(&event.after)
            {
                self.state.known_hand.push(event.after);
            }
        }
    }
    pub(super) fn reveal_top_draw_if_artifact(&mut self, item: &SpellView) {
        let player = item.controller;
        let Some(&id) = self.state.players[player.index()].library.last() else {
            return;
        };
        let revealed = RevealedLibraryCard {
            object: self.object_ref(id),
            card: self.state.objects[id as usize].card,
        };
        self.remember_library_top(player, revealed.object);
        // Reveal is public even if this ordinary draw completes within one action.
        let sequence = self
            .state
            .library_top
            .as_ref()
            .and_then(|s| s.reveal.as_ref())
            .map_or(0, |e| e.sequence)
            .checked_add(1)
            .capacity("library reveal sequence exhausted");
        let mut event = LibraryRevealEvent {
            sequence,
            source: item.source.expect("library effect requires source").object,
            player,
            revealed,
            drawn: None,
        };
        if revealed.card.kind_mask() & 16 != 0 {
            self.draw_many(player, 1);
            let result = self.object_ref(id);
            if result != revealed.object && self.state.objects[id as usize].zone == Zone::Hand {
                event.drawn = Some(result);
                if !self.state.known_hand.contains(&result) {
                    self.state.known_hand.push(result);
                }
            }
        }
        self.state.library_top.get_or_insert_default().reveal = Some(event);
    }
    pub(super) fn exile_top_play_boost_source(
        &mut self,
        item: &SpellView,
        subtype: &str,
        power: i32,
        toughness: i32,
    ) {
        let Some(&top) = self.state.players[item.controller.index()].library.last() else {
            return;
        };
        let events = self.move_objects_raw(&[top], Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(item.controller, event.after, false);
                if self.state.objects[event.after.card as usize]
                    .card
                    .has_subtype(subtype)
                    && self.on_battlefield(item.source.expect("library effect requires source").id)
                    && self.object_ref(item.source.expect("library effect requires source").id) == item.source.expect("library effect requires source").object
                {
                    self.state.objects[item.source.expect("library effect requires source").id as usize].growth += Stats {
                        power: i128::from(power),
                        toughness: i128::from(toughness),
                    };
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }
}
