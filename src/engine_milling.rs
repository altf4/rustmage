//! Batch milling and exact-incarnation, non-targeted recovery continuations.
use super::*;

impl Game {
    pub(super) fn mill_batch(
        &mut self,
        players: &[Player],
        amount: MillAmount,
        controller: Player,
    ) {
        // Freeze all players' top cards before moving anything. Mandatory milling
        // does as much as possible and never attempts a draw from an empty library.
        let maximum = match amount {
            MillAmount::LifeEvent => usize::try_from(
                self.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .life_event
                    .expect("saved life event")
                    .amount
                    .max(0),
            )
            .unwrap_or(usize::MAX),
            MillAmount::Fixed(n) => usize::from(n),
            MillAmount::ChosenX => {
                usize::try_from(self.state.resolving.as_ref().unwrap().item.x.unwrap_or(0))
                    .unwrap_or(usize::MAX)
            }
            MillAmount::ControllerLife => {
                usize::try_from(self.state.players[controller.index()].life.max(0))
                    .unwrap_or(usize::MAX)
            }
        };
        let ids: Vec<_> = players
            .iter()
            .flat_map(|player| {
                self.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .copied()
                    .filter(|&id| self.state.objects[id as usize].is_card())
                    .take(maximum)
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|event| event.moved)
            .map(|event| event.after)
            .collect();
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn can_recover_milled(
        &self,
        player: Player,
        filter: RecoveryFilter,
        object: ObjectRef,
    ) -> bool {
        self.state
            .resolving
            .as_ref()
            .is_some_and(|resolution| resolution.milled.contains(&object))
            && self.public_milled_object(object)
            && self
                .state
                .objects
                .get(object.card as usize)
                .is_some_and(|card| card.owner == player && filter.matches(card.card))
    }

    pub(super) fn recover_milled(&mut self, object: Option<ObjectRef>) {
        // The legal-action visitor already checked membership, generation, zone,
        // ownership and quality. Declining consumes this choice, without targeting.
        if let Some(object) = object {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            self.move_milled_selection(player, &[object], MilledDestination::Hand);
        }
        self.run_program();
    }
}
