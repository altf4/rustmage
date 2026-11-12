//! Completed bending actions and independent per-player turn history.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) enum BendKind {
    Air = 1,
    Earth = 2,
    Fire = 4,
    Water = 8,
}

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
    pub(super) fn register_bend_source(&mut self, id: CardId) {
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|ability| {
                matches!(
                    ability.condition.event(),
                    TriggerCondition::BendsYou | TriggerCondition::OwnAttackerAbilityTriggered
                ) && !ability.condition.is_granted()
                    && ability.condition.origin() != Some(TargetZone::Graveyard)
            })
        {
            self.state.bend_sources.get_or_insert_default().push(id);
        }
    }

    pub(super) fn forget_bend_source(&mut self, id: CardId) {
        if let Some(sources) = &mut self.state.bend_sources {
            sources.retain(|source| *source != id);
            if sources.is_empty() {
                self.state.bend_sources = None;
            }
        }
    }

    pub(super) fn record_bend(&mut self, actor: Player, kind: BendKind) {
        self.state.players[actor.index()].turn_facts.bent_elements |= kind as u8;
        let Some(sources) = self.state.bend_sources.clone() else {
            return;
        };
        for &id in sources.iter() {
            if self.state.objects[id as usize].controller != actor {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::BendsYou {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
