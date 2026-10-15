//! Private qualified library searches, owned selections and public hand knowledge.
use super::*;
use crate::SearchDestination;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SearchChoice {
    pub spec: SearchSpec,
    pub selected: Vec<CardId>,
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

    pub(super) fn finish_search(&mut self, player: Player) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .take()
            .unwrap();
        let destination = match choice.spec.destination {
            SearchDestination::Hand => Zone::Hand,
            SearchDestination::Battlefield => Zone::Battlefield,
        };
        let moves = choice
            .selected
            .iter()
            .map(|&id| {
                let mut movement = self.entry_move(
                    id,
                    destination,
                    destination == Zone::Battlefield
                        && (choice.spec.tapped || self.enters_tapped(id)),
                );
                movement.reveal_hand = choice.spec.reveal && destination == Zone::Hand;
                movement
            })
            .collect();
        self.stage_entries(moves, Some(player));
        self.run_program();
    }
}
