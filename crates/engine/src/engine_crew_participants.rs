//! Committed Crew/Saddle contribution events retain post-tap effective abilities.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn capture_crew_saddle_participants(
        &self,
        selected: &[ObjectRef],
    ) -> Vec<(ObjectView, Option<AttachmentTriggerView>)> {
        let has_event = |card: Card| {
            card.triggers()
                .iter()
                .any(|ability| ability.condition.event() == TriggerCondition::SelfCrewsOrSaddles)
        };
        let mut ids: Vec<_> = selected
            .iter()
            .filter(|object| has_event(self.state.objects[object.card as usize].card))
            .map(|object| object.card)
            .collect();
        // CR603.10 uses post-event characteristics. A participant can have a
        // real trigger granted by an attachment; freeze its effective post-tap
        // availability and grant origin before later costs or responses change it.
        for &id in &self.state.attachment_sources {
            if has_event(self.state.objects[id as usize].card)
                && !ids.contains(&id)
                && self
                    .attachment_recipient(id)
                    .is_some_and(|recipient| selected.contains(&recipient))
            {
                ids.push(id);
            }
        }
        self.indexed_trigger_sources(&ids)
            .into_iter()
            .filter(|(source, _)| selected.contains(&source.reference))
            .filter(|(source, grant)| {
                grant
                    .as_ref()
                    .map_or(source.card, |grant| grant.granting_card)
                    .triggers()
                    .iter()
                    .any(|ability| {
                        ability.condition.event() == TriggerCondition::SelfCrewsOrSaddles
                    })
            })
            .collect()
    }

    pub(super) fn queue_crew_saddle_participant_triggers(
        &mut self,
        sources: Vec<(ObjectView, Option<AttachmentTriggerView>)>,
        recipient: ObjectRef,
    ) {
        for (source, grant) in sources {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::SelfCrewsOrSaddles
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    // This is the exact non-targeted permanent whose activation
                    // was paid for, not the contributor and not an announced target.
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entering_object = Some(recipient);
                }
            }
        }
    }
}
