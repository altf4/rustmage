//! Individual successful draw events and globally tracked per-turn ordinals.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DrawTriggerView {
    pub player: Player,
    pub ordinal: u64,
}
impl Game {
    pub(super) fn card_drawn(&mut self, player: Player) {
        let count = &mut self.state.players[player.index()].cards_drawn_this_turn;
        *count = count.checked_add(1).expect("draw ordinal exhausted");
        let event = DrawTriggerView {
            player,
            ordinal: *count,
        };
        let sources: Vec<_> = self
            .state
            .draw_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::CardDrawn { relation, ordinal } = ability.condition else {
                    continue;
                };
                if ordinal.is_none_or(|n| n == event.ordinal)
                    && match relation {
                        CastRelation::You => source.owner == player,
                        CastRelation::Opponent => source.owner != player,
                        CastRelation::Any => true,
                    }
                {
                    self.queue_trigger(&source, index);
                    self.state.pending_triggers.last_mut().unwrap().draw_event = Some(event);
                }
            }
        }
    }
}
