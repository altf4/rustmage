//! Private qualified library searches, owned selections and public hand knowledge.
use super::*;
use crate::SearchDestination;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SearchChoice {
    pub spec: SearchSpec,
    pub minimum: u16,
    pub different_names: bool,
    pub selected: Vec<CardId>,
    pub battlefield: Option<Vec<CardId>>,
}

impl Game {
    pub(super) fn resolution_target_controller(&self) -> Option<Player> {
        let resolution = self.state.resolving.as_ref()?;
        resolution
            .target_card_facts
            .as_ref()?
            .iter()
            .find(|fact| fact.group == resolution.target_group)
            .map(|fact| fact.controller)
    }

    pub(super) fn choose_search(&mut self, card: CardId) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        choice.selected.push(card);
        // Selections remain private and in the library until FinishSearch. Even
        // a maximum-sized selection has an explicit commit boundary for replay.
    }

    pub(super) fn choose_search_battlefield(&mut self, player: Player, card: CardId) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        let SearchDestination::BattlefieldThenHand { battlefield } = choice.spec.destination else {
            unreachable!("validated split-search action")
        };
        let chosen = choice.battlefield.as_mut().unwrap();
        chosen.push(card);
        if chosen.len() == usize::from(battlefield) {
            self.commit_search(player);
        }
    }

    pub(super) fn finish_search(&mut self, player: Player) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        if let SearchDestination::BattlefieldThenHand { battlefield } = choice.spec.destination {
            if battlefield == 0 {
                choice.battlefield = Some(Vec::new());
            } else if choice.selected.len() > usize::from(battlefield) {
                // The selected identities have now been revealed; the rest of the
                // library stays private while the controller assigns destinations.
                choice.battlefield = Some(Vec::new());
                return;
            }
        }
        self.commit_search(player);
    }

    fn commit_search(&mut self, player: Player) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .take()
            .unwrap();
        if choice.spec.destination == SearchDestination::Top {
            // Searching does not change the found card's zone or incarnation.
            // Reveal and shuffle are one committed action; only the final known
            // top is public, never the rest of the private search/selection.
            if self.state.players[player.index()].library.len() > 1 {
                self.forget_library_top(player);
            }
            let library = &mut self.state.players[player.index()].library;
            self.state.rng.shuffle(library);
            for &id in &choice.selected {
                let position = library.iter().position(|&card| card == id).unwrap();
                library.remove(position);
                library.push(id);
            }
            if choice.spec.reveal
                && let Some(&id) = choice.selected.last()
            {
                self.remember_library_top(player, self.object_ref(id));
            }
            self.run_program();
            return;
        }
        // A private search does not disclose whether the previously known top
        // was chosen or shuffled away. A singleton library is still deducible.
        if choice.spec.destination == SearchDestination::Hand
            && !choice.spec.reveal
            && self.state.players[player.index()].library.len() > 1
        {
            self.forget_library_top(player);
        }
        let moves = choice
            .selected
            .iter()
            .map(|&id| {
                let destination = match choice.spec.destination {
                    SearchDestination::Top => {
                        unreachable!("top search committed before entry moves")
                    }
                    SearchDestination::LinkedExile => Zone::Exile,
                    SearchDestination::Hand => Zone::Hand,
                    SearchDestination::Graveyard => Zone::Graveyard,
                    SearchDestination::Battlefield => Zone::Battlefield,
                    SearchDestination::BattlefieldThenHand { .. } => {
                        if choice
                            .battlefield
                            .as_ref()
                            .is_none_or(|selected| selected.contains(&id))
                        {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    }
                };
                let mut movement = self.entry_move(
                    id,
                    destination,
                    destination == Zone::Battlefield && choice.spec.tapped,
                );
                movement.reveal_hand = choice.spec.reveal && destination == Zone::Hand;
                movement
            })
            .collect();
        self.stage_entries(moves, Some(player));
        if choice.spec.destination == SearchDestination::LinkedExile {
            self.remember_linked_exile(&choice.selected);
        }
        self.run_program();
    }
}

/// Public saved recipients: independent of the land/source after destruction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OptionalSearchSequence {
    pub players: [Player; 2],
    pub next: usize,
}
impl Game {
    pub(super) fn destroy_land_and_search(&mut self, item: &SpellView) {
        let Some(bound) = item
            .targets
            .first()
            .copied()
            .filter(|bound| self.same_target_object(*bound))
        else {
            return;
        };
        let Target::Card(id) = bound.target else {
            return;
        };
        let affected = self.state.objects[id as usize].controller;
        self.state.resolving.as_mut().unwrap().optional_searches =
            Some(Box::new(OptionalSearchSequence {
                players: [affected, item.controller],
                next: 0,
            }));
        if !self.keywords(id).contains(Keywords::INDESTRUCTIBLE) {
            self.move_target_batch(&[bound], Zone::Graveyard);
        }
    }
    pub(super) fn advance_optional_searches(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(sequence) = resolution.optional_searches.as_ref() else {
            return false;
        };
        if resolution.search.is_some() {
            return false;
        }
        if let Some(&player) = sequence.players.get(sequence.next) {
            self.state.pending = Pending::OptionalSearch(player);
            true
        } else {
            resolution.optional_searches = None;
            false
        }
    }
    pub(super) fn choose_optional_search(&mut self, player: Player, accept: bool) {
        let resolution = self.state.resolving.as_mut().unwrap();
        resolution.optional_searches.as_mut().unwrap().next += 1;
        if accept {
            resolution.search = Some(Box::new(SearchChoice {
                spec: SearchSpec {
                    filter: crate::SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped: false,
                    reveal: false,
                },
                minimum: 0,
                different_names: false,
                selected: Vec::new(),
                battlefield: None,
            }));
            self.state.pending = Pending::EffectSearch { player };
        } else {
            self.run_program();
        }
    }
}
