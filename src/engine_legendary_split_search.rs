//! Private mixed-zone qualified search, followed by a public opponent choice.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LegendarySplitSearch {
    controller: Player,
    maximum: u16,
    hand: u16,
    pub revealed: bool,
    found: Vec<ObjectRef>,
    chosen: Vec<ObjectRef>,
}
impl LegendarySplitSearch {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.found.len() + self.chosen.len()) * std::mem::size_of::<ObjectRef>()
    }
    fn hand_count(&self) -> usize {
        usize::from(self.hand).min(self.found.len())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct LegendarySplitSearchView {
    pub controller: Player,
    pub revealed: bool,
    pub library: Vec<ObjectView>,
    pub outside: Vec<ObjectView>,
    pub candidates: Vec<ObjectRef>,
    pub found: Vec<ObjectView>,
    pub chosen_for_hand: Vec<ObjectRef>,
    pub maximum: u16,
    pub required_for_hand: usize,
    pub can_finish: bool,
}
impl Game {
    pub(super) fn start_legendary_split_search(
        &mut self,
        controller: Player,
        maximum: u16,
        hand: u16,
    ) {
        self.state.resolving.as_mut().unwrap().legendary_search =
            Some(Box::new(LegendarySplitSearch {
                controller,
                maximum,
                hand,
                revealed: false,
                found: Vec::new(),
                chosen: Vec::new(),
            }));
        self.state.pending = Pending::LegendarySplitSearch { player: controller };
    }
    fn legendary_split_card(&self, controller: Player, object: ObjectRef) -> bool {
        let Some(card) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        self.object_ref(object.card) == object
            && !card.is_token()
            && card.owner == controller
            && card.card.is_creature()
            && card.card.is_legendary()
            && (card.zone == Zone::Library || self.outside_game_card_matches(controller, object))
    }
    pub(super) fn legendary_split_candidates(&self, player: Player) -> Vec<ObjectRef> {
        let search = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .legendary_search
            .as_ref()
            .unwrap();
        if search.revealed {
            if player != search.controller.other() || search.chosen.len() >= search.hand_count() {
                return Vec::new();
            }
            return search
                .found
                .iter()
                .copied()
                .filter(|object| {
                    self.legendary_split_card(search.controller, *object)
                        && !search.chosen.contains(object)
                })
                .collect();
        }
        if player != search.controller || search.found.len() >= usize::from(search.maximum) {
            return Vec::new();
        }
        self.state.players[player.index()]
            .library
            .iter()
            .map(|&id| self.object_ref(id))
            .chain(self.outside_game_cards(player))
            .filter(|object| {
                self.legendary_split_card(player, *object)
                    && !search.found.iter().any(|prior| {
                        crate::card_names::same_name(
                            self.object_name(prior.card),
                            self.object_name(object.card),
                        )
                    })
            })
            .collect()
    }
    pub(super) fn legendary_split_can_finish(&self) -> bool {
        let search = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .legendary_search
            .as_ref()
            .unwrap();
        !search.revealed || search.chosen.len() == search.hand_count()
    }
    pub(super) fn choose_legendary_split_card(&mut self, card: CardId) {
        let object = self.object_ref(card);
        let search = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .legendary_search
            .as_mut()
            .unwrap();
        if search.revealed {
            search.chosen.push(object);
        } else {
            search.found.push(object);
        }
    }
    pub(super) fn finish_legendary_split_search(&mut self) {
        let search = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .legendary_search
            .as_mut()
            .unwrap();
        if !search.revealed {
            // CR701.23b permits failing to find some/all qualified hidden cards.
            // Unlike pinned Java, partial found sets continue through later instructions.
            search.revealed = true;
            if search.hand_count() != 0 {
                self.state.pending = Pending::LegendarySplitSearch {
                    player: search.controller.other(),
                };
                return;
            }
        }
        let search = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .legendary_search
            .take()
            .unwrap();
        let moves = search
            .found
            .iter()
            .filter(|object| {
                search.chosen.contains(object)
                    || self.state.objects[object.card as usize].zone != Zone::Library
            })
            .map(|object| {
                let hand = search.chosen.contains(object);
                let mut movement = self.entry_move(
                    object.card,
                    if hand { Zone::Hand } else { Zone::Library },
                    false,
                );
                movement.reveal_hand = hand;
                movement
            })
            .collect();
        self.stage_entries(moves, Some(search.controller));
        self.run_program();
    }
    pub(super) fn legendary_split_view(&self, viewer: Player) -> Option<LegendarySplitSearchView> {
        let search = self.state.resolving.as_ref()?.legendary_search.as_ref()?;
        if !search.revealed && viewer != search.controller {
            return None;
        }
        Some(LegendarySplitSearchView {
            controller: search.controller,
            revealed: search.revealed,
            library: if search.revealed {
                Vec::new()
            } else {
                self.state.players[search.controller.index()]
                    .library
                    .iter()
                    .map(|&id| self.object_view(id))
                    .collect()
            },
            outside: if search.revealed {
                Vec::new()
            } else {
                self.outside_game_cards(search.controller)
                    .map(|object| self.object_view(object.card))
                    .collect()
            },
            candidates: self.legendary_split_candidates(if search.revealed {
                search.controller.other()
            } else {
                search.controller
            }),
            found: search
                .found
                .iter()
                .map(|object| self.object_view(object.card))
                .collect(),
            chosen_for_hand: search.chosen.clone(),
            maximum: search.maximum,
            required_for_hand: search.hand_count(),
            can_finish: self.legendary_split_can_finish(),
        })
    }
}
