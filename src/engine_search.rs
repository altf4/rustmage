//! Private qualified library searches, owned selections and public hand knowledge.
use super::*;
use crate::SearchDestination;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SearchChoice {
    pub spec: SearchSpec,
    pub minimum: u16,
    pub selected: Vec<CardId>,
    pub battlefield: Option<Vec<CardId>>,
}

impl Game {
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
        self.run_program();
    }
}
