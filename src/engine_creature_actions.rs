//! Exact non-targeted attack/block event subjects; later incarnations never inherit a packet.
use super::*;
impl Game {
    pub(super) fn creature_action_triggers(
        &mut self,
        actor: Player,
        creatures: &[CardId],
        kind: CreatureActionKind,
    ) {
        if creatures.is_empty() {
            return;
        }
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|id| self.on_battlefield(*id))
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            if source.controller != actor {
                continue;
            }
            let definition = source.card;
            for (index, ability) in definition.triggers().iter().enumerate() {
                let qualifies = match ability.condition.event() {
                    TriggerCondition::CreatureAttacksOrBlocksYou => true,
                    TriggerCondition::ControlledCreatureAttacksAlone => {
                        kind == CreatureActionKind::Attack && creatures.len() == 1
                    }
                    _ => false,
                };
                if !qualifies {
                    continue;
                }
                for &id in creatures {
                    if self.queue_trigger(&source, index) {
                        let object = self.object_ref(id);
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .creature_action_event =
                            Some(Box::new(CreatureActionEventView { object, kind }));
                    }
                }
            }
        }
    }
    pub(super) fn grow_creature_action_scaled(&mut self, item: &SpellView, value: AmountValue) {
        let Some(event) = item.creature_action_event.as_ref() else {
            return;
        };
        let id = event.object.card;
        if !self.on_battlefield(id) || self.object_ref(id) != event.object || !self.is_creature(id)
        {
            return;
        }
        let amount = i128::try_from(self.item_amount(value, item))
            .capacity("creature growth amount exhausted");
        self.state.objects[id as usize].growth += Stats {
            power: amount,
            toughness: amount,
        };
    }
    pub(super) fn grow_creature_action_difference(&mut self, item: &SpellView) {
        let Some(event) = item.creature_action_event.as_ref() else {
            return;
        };
        let id = event.object.card;
        if !self.on_battlefield(id) || self.object_ref(id) != event.object || !self.is_creature(id)
        {
            return;
        }
        let stats = self.stats(id);
        let amount = i128::try_from(stats.power.abs_diff(stats.toughness))
            .capacity("power/toughness difference exhausted");
        self.state.objects[id as usize].growth += Stats {
            power: amount,
            toughness: amount,
        };
    }
}
