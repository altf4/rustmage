//! Individual successful draw events and globally tracked per-turn ordinals.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DrawTriggerView {
    pub player: Player,
    pub ordinal: u64,
}
impl Game {
    pub(super) fn quantum_draw_sources(&self, player: Player) -> Vec<ObjectRef> {
        if self.hand_count(player) > 1 {
            return Vec::new();
        }
        self.state
            .static_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.state.objects[id as usize].controller == player
                    && self.state.objects[id as usize].card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::AdditionalDrawWithAtMostOneCard)))
                    && self.continuous_fragment_available(id, |fragment| {
                        matches!(fragment, crate::cards::StaticAbilityFragment::AdditionalDrawWithAtMostOneCard)
                    })
            })
            .map(|id| self.object_ref(id))
            .collect()
    }
    pub(super) fn doubled_draw_sources(
        &self,
        player: Player,
    ) -> impl Iterator<Item = ObjectRef> + '_ {
        let exempt = self.state.step == Step::Draw
            && self.state.active == player
            && !self.state.players[player.index()].drew_in_own_draw_step;
        self.state
            .static_sources
            .iter()
            .copied()
            .filter(move |&id| {
                !exempt
                    && self.on_battlefield(id)
                    && self.state.objects[id as usize].controller == player
                    && self.state.objects[id as usize]
                        .card
                        .static_ability_groups()
                        .iter()
                        .any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    crate::cards::StaticAbilityFragment::DoubleDrawExceptFirst
                                )
                            })
                        })
                    && self.continuous_fragment_available(id, |fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::DoubleDrawExceptFirst
                        )
                    })
            })
            .map(|id| self.object_ref(id))
    }
    pub(super) fn replaced_draw_count_excluding(
        &self,
        player: Player,
        excluded: &[ObjectRef],
    ) -> u128 {
        self.doubled_draw_sources(player)
            .into_iter()
            .filter(|source| !excluded.contains(source))
            .fold(1u128, |count, _| {
                count
                    .checked_mul(2)
                    .capacity("draw replacement quantity exhausted")
            })
    }

    pub(super) fn card_drawn(&mut self, player: Player) {
        if self.state.step == Step::Draw && self.state.active == player {
            self.state.players[player.index()].drew_in_own_draw_step = true;
        }
        let count = &mut self.state.players[player.index()].cards_drawn_this_turn;
        *count = count.checked_add(1).capacity("draw ordinal exhausted");
        let event = DrawTriggerView {
            player,
            ordinal: *count,
        };
        let sources = self.indexed_trigger_sources(&self.state.draw_sources);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let TriggerCondition::CardDrawn { relation, ordinal } = ability.condition.event()
                else {
                    continue;
                };
                if ordinal.is_none_or(|n| n == event.ordinal)
                    && match relation {
                        CastRelation::You => source.controller == player,
                        CastRelation::Opponent => source.controller != player,
                        CastRelation::Any => true,
                    }
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state.pending_triggers.last_mut().unwrap().draw_event = Some(event);
                }
            }
        }
    }
}
