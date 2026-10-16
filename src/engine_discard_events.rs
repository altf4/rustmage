//! Cause-specific simultaneous discards and snapshot-owned trigger quantities.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DiscardTriggerView {
    pub player: Player,
    pub amount: u32,
    pub cycling: bool,
}

impl Game {
    pub(super) fn discard_batch(&mut self, cards: &[CardId], cycled: Option<ObjectRef>) {
        if cards.is_empty() {
            return;
        }
        assert!(
            cards
                .iter()
                .all(|&id| self.state.objects[id as usize].zone == Zone::Hand)
        );
        let sources: Vec<_> = self
            .state
            .discard_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        let events = self.move_objects_raw(cards, Zone::Graveyard);
        self.finish_zone_events(&events, &[]);
        let mut counts = [0u32; 2];
        for event in &events {
            if !event.moved {
                continue;
            }
            let player = event.before.owner;
            counts[player.index()] = counts[player.index()]
                .checked_add(1)
                .capacity("discard count exhausted");
            let cycling = cycled == Some(event.before.reference);
            // Self-discard/cycle triggers use the card in its destination zone.
            let source = self.object_view(event.after.card);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::SelfDiscarded
                    || cycling && ability.condition.event() == TriggerCondition::SelfCycled
                {
                    self.queue_discard_trigger(
                        &source,
                        index,
                        DiscardTriggerView {
                            player,
                            amount: 1,
                            cycling,
                        },
                    );
                }
            }
        }
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::Discarded {
                    relation,
                    one_or_more,
                } = ability.condition.event()
                else {
                    continue;
                };
                for player in [self.state.active, self.state.active.other()] {
                    let count = counts[player.index()];
                    if count == 0
                        || !match relation {
                            CastRelation::You => source.owner == player,
                            CastRelation::Opponent => source.owner != player,
                            CastRelation::Any => true,
                        }
                    {
                        continue;
                    }
                    let cycling = cycled.is_some_and(|reference| {
                        self.state.objects[reference.card as usize].owner == player
                    });
                    for _ in 0..if one_or_more { 1 } else { count } {
                        self.queue_discard_trigger(
                            &source,
                            index,
                            DiscardTriggerView {
                                player,
                                amount: if one_or_more { count } else { 1 },
                                cycling,
                            },
                        );
                    }
                }
            }
        }
    }
    fn queue_discard_trigger(
        &mut self,
        source: &ObjectView,
        index: usize,
        event: DiscardTriggerView,
    ) {
        if !self.queue_trigger(source, index) {
            return;
        }
        self.state
            .pending_triggers
            .last_mut()
            .unwrap()
            .discard_event = Some(event);
    }
}
