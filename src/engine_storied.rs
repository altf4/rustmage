//! Storied is a continuous player designation, acquired before triggers or SBAs.
//! Real and funding states own independent persistent bits; characteristic queries read them.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;

/// Pure acquisition check shared by actual state and realizable funding worlds.
/// Existing designations are read from the world, never from a cached real-game
/// ability evaluation. The caller commits only to the world it owns.
pub(super) fn acquisitions<W: CharacteristicWorld + ?Sized>(world: &W) -> [bool; 2] {
    if [Player::P0, Player::P1]
        .into_iter()
        .all(|p| world.has_enduring_story(p))
    {
        return [false; 2];
    }
    let storied = |id: CardId| {
        world
            .object(id)
            .card
            .static_ability_groups()
            .iter()
            .any(|group| {
                group
                    .fragments
                    .iter()
                    .any(|fragment| matches!(fragment, StaticAbilityFragment::Storied))
            })
    };
    let mut candidates = [false; 2];
    for &id in world.static_sources() {
        let object = world.object(id);
        if object.zone == Zone::Battlefield
            && !world.has_enduring_story(object.controller)
            && storied(id)
        {
            candidates[object.controller.index()] = true;
        }
    }
    if !candidates.into_iter().any(|candidate| candidate) {
        return [false; 2];
    }
    let query = characteristic_query::CharacteristicQuery::in_world(world);
    let mut acquired = [false; 2];
    for player in [Player::P0, Player::P1] {
        if !candidates[player.index()] {
            continue;
        }
        let live = world.static_sources().iter().any(|&id| {
            let object = world.object(id);
            object.zone == Zone::Battlefield
                && object.controller == player
                && storied(id)
                && query.continuous_fragment_available(id, |fragment| {
                    matches!(fragment, StaticAbilityFragment::Storied)
                })
        });
        if live {
            acquired[player.index()] = world
                .game()
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, _)| {
                    let id = *id as CardId;
                    let object = world.object(id);
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
}

impl Game {
    pub(super) fn refresh_enduring_stories(&mut self) {
        let acquired = acquisitions(self);
        for (player, acquired) in self.state.players.iter_mut().zip(acquired) {
            player.enduring_story |= acquired;
        }
    }
}
