//! Expend observes actual mana payments during casting (CR 700.14).
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Sources = Option<SharedVec<CardId>>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(sources: &Sources) -> usize {
    sources.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CardId>>() + rows.len() * std::mem::size_of::<CardId>()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn register_expend_source(&mut self, id: CardId) {
        if self.has_trigger(id, |ability| {
            ability.condition.origin() != Some(TargetZone::Graveyard)
                && !ability.condition.is_granted()
                && matches!(ability.condition.event(), TriggerCondition::Expend(_))
        }) {
            let sources = self.state.expend_sources.get_or_insert_default();
            if !sources.contains(&id) {
                sources.push(id);
            }
        }
    }

    pub(super) fn forget_expend_source(&mut self, id: CardId) {
        if let Some(sources) = &mut self.state.expend_sources {
            sources.retain(|source| *source != id);
            if sources.is_empty() {
                self.state.expend_sources = None;
            }
        }
    }

    pub(super) fn record_expend(&mut self, actor: Player, payment: Mana) {
        let spent = payment.into_iter().map(u64::from).sum::<u64>();
        let total = &mut self.state.players[actor.index()].spell_mana_spent_this_turn;
        let before = *total;
        let after = before.checked_add(spent).capacity("expend total exhausted");
        *total = after;
        let Some(sources) = self.state.expend_sources.clone() else {
            return;
        };
        // A later cost can remove the observer. Queue at payment and let the
        // normal casting transaction deliver the trigger once priority resumes.
        for &id in sources.iter() {
            if self.state.objects[id as usize].controller != actor {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.trigger_templates() {
                if let TriggerCondition::Expend(threshold) = ability.condition.event()
                    && before < threshold
                    && threshold <= after
                {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
