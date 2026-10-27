//! Target acquisition is an actual event; queries and provisional edits stay pure.
use super::*;
pub(super) type Sources = Option<SharedVec<CardId>>;
pub(super) fn owned_bytes(sources: &Sources) -> usize {
    sources.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CardId>>() + rows.len() * std::mem::size_of::<CardId>()
    })
}
impl Game {
    pub(super) fn register_ability_target_source(&mut self, id: CardId) {
        if self.state.objects[id as usize]
            .card
            .triggers()
            .iter()
            .any(|a| {
                matches!(
                    a.condition.event(),
                    TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent
                        | TriggerCondition::OpponentTargetsControlledCreatureOrSpell
                )
            })
        {
            let sources = self.state.ability_target_sources.get_or_insert_default();
            if !sources.contains(&id) {
                sources.push(id);
            }
        }
    }
    pub(super) fn forget_ability_target_source(&mut self, id: CardId) {
        if let Some(sources) = &mut self.state.ability_target_sources {
            sources.retain(|&source| source != id);
            if sources.is_empty() {
                self.state.ability_target_sources = None;
            }
        }
    }
    pub(super) fn controlled_ability_target_acquired(
        &mut self,
        item: &SpellView,
        bound: BoundTarget,
    ) {
        if item.kind == StackKind::Spell
            || item
                .targets
                .iter()
                .any(|old| old.target == bound.target && old.object == bound.object)
        {
            return;
        }
        let qualified = match bound.target {
            Target::Player(_) => true,
            Target::Card(id) => bound
                .object
                .is_some_and(|object| self.on_battlefield(id) && self.object_ref(id) == object),
            Target::Spell(_) | Target::Stack(_) => false,
        };
        if !qualified {
            return;
        }
        let sources: Vec<_> = self
            .state
            .ability_target_sources
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|&&id| {
                self.on_battlefield(id)
                    && self.state.objects[id as usize].controller == item.controller
            })
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event()
                    == TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent
                {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
    /// Opponent spell/ability targeting a controlled creature permanent or spell.
    pub(super) fn opponent_targets_controlled_creature(
        &mut self,
        item: &SpellView,
        bound: BoundTarget,
    ) {
        if item
            .targets
            .iter()
            .any(|old| old.target == bound.target && old.object == bound.object)
        {
            return;
        }
        let target_controller = match bound.target {
            Target::Card(id)
                if bound.object.is_some_and(|r| {
                    self.on_battlefield(id) && self.object_ref(id) == r && self.is_creature(id)
                }) =>
            {
                Some(self.state.objects[id as usize].controller)
            }
            Target::Spell(_) | Target::Stack(_) => self
                .bound_stack_item(&bound)
                .filter(|target| {
                    target.kind == StackKind::Spell && target.source_card.is_creature()
                })
                .map(|target| target.controller),
            _ => None,
        };
        let Some(controller) = target_controller.filter(|&p| p != item.controller) else {
            return;
        };
        let sources: Vec<_> = self
            .state
            .ability_target_sources
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|&&id| {
                self.on_battlefield(id) && self.state.objects[id as usize].controller == controller
            })
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event()
                    == TriggerCondition::OpponentTargetsControlledCreatureOrSpell
                {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
    /// Commit only final entity acquisitions, independent of target group/slot.
    /// A new copied ability uses an empty previous set, exactly like a new spell.
    pub(super) fn acquire_stack_target_set(&mut self, item: &SpellView, previous: &[BoundTarget]) {
        let mut before = item.clone();
        before.targets = previous.to_vec();
        for bound in &item.targets {
            if !before
                .targets
                .iter()
                .any(|old| old.target == bound.target && old.object == bound.object)
            {
                self.target_becomes_bound(&before, *bound);
                before.targets.push(*bound);
            }
        }
    }
}
