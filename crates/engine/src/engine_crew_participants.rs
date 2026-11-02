//! Committed Crew/Saddle contribution events retain pre-tap effective abilities.
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
        // A participant can have a real trigger granted by an attachment. Capture
        // both the recipient's effective availability and the grant's origin now.
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
        for (mut source, grant) in sources {
            // The event is the completed tap cost. Availability was captured
            // before that simultaneous transition; tapped facts describe it.
            source.tapped = true;
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
