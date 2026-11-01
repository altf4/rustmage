//! Batch milling and exact-incarnation, non-targeted recovery continuations.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn mill_batch(
        &mut self,
        players: &[Player],
        amount: MillAmount,
        controller: Player,
    ) -> usize {
        // Freeze all players' top cards before moving anything. Mandatory milling
        // does as much as possible and never attempts a draw from an empty library.
        if amount == MillAmount::RecipientGraveyard {
            return self
                .mill_batch_with_counts(players, |game, player| {
                    game.state
                        .objects
                        .iter()
                        .filter(|object| {
                            object.zone == Zone::Graveyard
                                && object.owner == player
                                && object.is_card()
                        })
                        .count()
                })
                .iter()
                .filter(|event| event.moved || event.shuffled_in_place)
                .count();
        }
        let maximum = match amount {
            MillAmount::RecipientGraveyard => unreachable!("per-player counts handled above"),
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
        self.mill_batch_up_to(players, maximum)
            .iter()
            .filter(|event| event.moved || event.shuffled_in_place)
            .count()
    }

    /// The rad-trigger continuation uses the exact committed destinations and
    /// public characteristics of this one mill event, including replacements.
    pub(super) fn mill_radiation(&mut self, player: Player, count: u64) -> u64 {
        let maximum = usize::try_from(count).unwrap_or(usize::MAX);
        let events = self.mill_batch_up_to(&[player], maximum);
        u64::try_from(
            events
                .iter()
                .filter(|event| {
                    event.moved
                        && self.public_milled_object(event.after)
                        && self
                            .state
                            .objects
                            .get(event.after.card as usize)
                            .is_some_and(|object| {
                                object.generation == event.after.generation
                                    && !object.is_token()
                                    && !object.is_spell_copy()
                                    && self.current_kinds(event.after.card) & 2 == 0
                            })
                })
                .count(),
        )
        .capacity("radiation mill count exhausted")
    }

    fn mill_batch_up_to(&mut self, players: &[Player], maximum: usize) -> Vec<ZoneChange> {
        self.mill_batch_with_counts(players, |_, _| maximum)
    }

    fn mill_batch_with_counts(
        &mut self,
        players: &[Player],
        maximum: impl Fn(&Self, Player) -> usize,
    ) -> Vec<ZoneChange> {
        let ids: Vec<_> = players
            .iter()
            .flat_map(|player| {
                self.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .copied()
                    .filter(|&id| self.state.objects[id as usize].is_card())
                    .take(maximum(self, *player))
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|event| event.moved)
            .map(|event| event.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        events
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
