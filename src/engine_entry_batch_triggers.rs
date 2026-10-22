//! Entry qualification belongs to one whole committed event, not later objects.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryBatchEventView {
    pub objects: Vec<ObjectRef>,
}
impl Game {
    pub(super) fn batch_graveyard_entry_triggers(
        &mut self,
        events: &[ZoneChange],
        sources: &[ObjectView],
        suppression: u16,
    ) {
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::CreaturesFromGraveyardEnterYou { other } =
                    ability.condition.event()
                else {
                    continue;
                };
                let objects: Vec<_> = events
                    .iter()
                    .filter_map(|event| {
                        if !event.moved
                            || event.destination != Zone::Battlefield
                            || (event.before.zone != Zone::Graveyard
                                && event.cast_origin != Some(Zone::Graveyard))
                            || (other && event.after == source.reference)
                            || !self.on_battlefield(event.after.card)
                            || self.object_ref(event.after.card) != event.after
                        {
                            return None;
                        }
                        let entered = self.object_view(event.after.card);
                        (entered.is_creature()
                            && entered.controller == source.controller
                            && entered.kinds & suppression == 0)
                            .then_some(event.after)
                    })
                    .collect();
                if !objects.is_empty() && self.queue_trigger(source, index) {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entry_batch_event = Some(Box::new(EntryBatchEventView { objects }));
                }
            }
        }
    }
}
