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
        let before: Vec<_> = choice
            .selected
            .iter()
            .map(|&id| {
                (
                    self.object_view(id),
                    destination == Zone::Battlefield
                        && (choice.spec.tapped || self.enters_tapped(id)),
                )
            })
            .collect();
        let events: Vec<_> = before
            .into_iter()
            .map(|(before, tapped)| {
                let mut event = self.move_object_raw_with_entry(before.id, destination, tapped);
                event.before = before;
                event
            })
            .collect();
        // Hand identities revealed by this search stay public knowledge until
        // they change zones. Hidden hand/library randomization must explicitly
        // invalidate such knowledge when that mechanic is implemented.
        if choice.spec.reveal && destination == Zone::Hand {
            self.state
                .known_hand
                .extend(events.iter().filter(|e| e.moved).map(|e| e.after));
        }
        self.finish_zone_events(&events, &[]);
        self.state
            .rng
            .shuffle(&mut self.state.players[player.index()].library);
        self.run_program();
    }
}
