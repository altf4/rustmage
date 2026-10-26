//! Storied is a continuous player designation, acquired before triggers or SBAs.
//! The two persistent bits are owned state. Queries/funding worlds only read them.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;

impl Game {
    pub(super) fn refresh_enduring_stories(&mut self) {
        if self
            .state
            .players
            .iter()
            .all(|player| player.enduring_story)
        {
            return;
        }
        // Reuse the existing sparse static-provider index. Ordinary games without
        // a storied provider never construct a characteristic evaluation here.
        let mut candidates = [false; 2];
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone == Zone::Battlefield
                && !self.state.players[object.controller.index()].enduring_story
                && object.card.static_ability_groups().iter().any(|group| {
                    group
                        .fragments
                        .iter()
                        .any(|fragment| matches!(fragment, StaticAbilityFragment::Storied))
                })
            {
                candidates[object.controller.index()] = true;
            }
        }
        if !candidates.into_iter().any(|candidate| candidate) {
            return;
        }
        let acquired = {
            let query = self.characteristic_query();
            let mut acquired = [false; 2];
            for player in [Player::P0, Player::P1] {
                if !candidates[player.index()] {
                    continue;
                }
                let live = self.state.static_sources.iter().any(|&id| {
                    let object = &self.state.objects[id as usize];
                    object.zone == Zone::Battlefield
                        && object.controller == player
                        && object.card.static_ability_groups().iter().any(|group| {
                            group
                                .fragments
                                .iter()
                                .any(|fragment| matches!(fragment, StaticAbilityFragment::Storied))
                        })
                        && query.continuous_fragment_available(id, |fragment| {
                            matches!(fragment, StaticAbilityFragment::Storied)
                        })
                });
                if live {
                    // The union is tested once per permanent, including tokens
                    // and lands; a legendary artifact Saga still counts once.
                    acquired[player.index()] = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            let id = *id as CardId;
                            object.zone == Zone::Battlefield
                                && object.controller == player
                                && (query.kinds(id) & 16 != 0
                                    || query.legendary(id)
                                    || query.subtypes(id).has(object.card, "Saga"))
                        })
                        .take(3)
                        .count()
                        == 3;
                }
            }
            acquired
        };
        // Commit after dropping all borrowed characteristic facts. No query
        // invokes a mutating refresh and no speculative world reaches this path.
        for (player, acquired) in self.state.players.iter_mut().zip(acquired) {
            player.enduring_story |= acquired;
        }
    }
}
