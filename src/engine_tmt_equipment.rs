//! Successful attachment reflexives and simultaneous combat-damage grants.
use super::*;
impl Game {
    pub(super) fn attach_source_reflexive(&mut self, item: &SpellView, ability: u16) {
        let Some(target) = item.targets.iter().find_map(|bound| bound.object) else {
            return;
        };
        if self.on_battlefield(item.card)
            && self.object_ref(item.card) == item.object
            && self.state.objects[item.card as usize].attached_to != Some(target)
            && self.attach_equipment(item.object, target)
        {
            self.queue_reflexive(item, ability, None);
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .source_attachment = Some(target);
        }
    }
    pub(super) fn equipment_combat_damage_triggers(&mut self, events: &[DamageEvent]) {
        let sources = self.indexed_trigger_sources(&self.state.damage_sources);
        for (source, grant) in sources {
            let Some(grant) = grant else { continue };
            if !events.iter().any(|event| {
                event.combat && event.amount > 0 && event.source.reference == source.reference
            }) {
                continue;
            }
            for (index, ability) in grant.granting_card.triggers().iter().enumerate() {
                if ability.condition.is_granted()
                    && matches!(
                        ability.condition.event(),
                        TriggerCondition::SourceDealtCombatDamage
                    )
                {
                    self.queue_trigger_from(&source, index, Some(Box::new(grant)));
                }
            }
        }
    }
}
