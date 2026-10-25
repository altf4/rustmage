//! Private looked-library free casts, with exact random remainder completion.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibraryCastLook {
    player: Player,
    greatest_power: i128,
    objects: Vec<ObjectRef>,
}
impl LibraryCastLook {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.objects.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct LibraryCastLookView {
    pub player: Player,
    pub greatest_attacking_power: i128,
    pub cards: Vec<ObjectView>,
}
impl Game {
    pub(super) fn cast_library_by_attacking_power(&mut self, item: &SpellView, amount: u16) {
        let player = item.controller;
        let greatest_power = self
            .state
            .attackers
            .iter()
            .filter(|a| {
                self.on_battlefield(a.card)
                    && self.is_creature(a.card)
                    && self.state.objects[a.card as usize].controller == player
            })
            .map(|a| self.stats(a.card).power)
            .max()
            .unwrap_or(0);
        let objects = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .take(usize::from(amount))
            .map(|&id| self.object_ref(id))
            .collect::<Vec<_>>();
        let maximum_mana_value = if greatest_power < 0 {
            Some(0)
        } else {
            u64::try_from(greatest_power)
                .ok()
                .and_then(|n| n.checked_add(1))
        };
        self.state
            .linked_choices
            .get_or_insert_default()
            .library_cast = Some(Box::new(LibraryCastLook {
            player,
            greatest_power,
            objects: objects.clone(),
        }));
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            casts_remaining: Some(1),
            pending_cast: None,
            alternative_cost: None,
            maximum_mana_value,
            origin: Zone::Library,
            exile_on_graveyard: false,
            allow_lands: false,
            player,
            remaining: objects,
        }));
        self.state.pending = Pending::ResolutionCasts { player };
    }
    pub(super) fn finish_library_cast_look(&mut self) {
        let Some(looked) = self
            .state
            .linked_choices
            .as_mut()
            .and_then(|s| s.library_cast.take())
        else {
            return;
        };
        let mut bottom = looked
            .objects
            .into_iter()
            .filter(|&r| {
                self.object_ref(r.card) == r
                    && self.state.objects[r.card as usize].zone == Zone::Library
                    && self.state.objects[r.card as usize].owner == looked.player
            })
            .map(|r| r.card)
            .collect::<Vec<_>>();
        self.state.rng.shuffle(&mut bottom);
        self.forget_library_top(looked.player);
        self.library_top_changed(looked.player);
        let library = &mut self.state.players[looked.player.index()].library;
        library.retain(|id| !bottom.contains(id));
        library.splice(0..0, bottom);
    }
    pub(super) fn library_cast_look_view(&self, viewer: Player) -> Option<LibraryCastLookView> {
        let looked = self.state.linked_choices.as_ref()?.library_cast.as_ref()?;
        if !self.shares_information(viewer, looked.player) {
            return None;
        }
        Some(LibraryCastLookView {
            player: looked.player,
            greatest_attacking_power: looked.greatest_power,
            cards: looked
                .objects
                .iter()
                .filter(|&&r| {
                    self.object_ref(r.card) == r
                        && self.state.objects[r.card as usize].zone == Zone::Library
                })
                .map(|r| self.object_view(r.card))
                .collect(),
        })
    }
}
