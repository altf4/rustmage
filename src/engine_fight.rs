//! Noncombat damage using current creature characteristics and distinct operands.
use super::*;
impl Game {
    pub(super) fn creature_power_damage(
        &mut self,
        item: &SpellView,
        source: u16,
        recipient: u16,
        fight: bool,
    ) {
        let creature = |group| {
            item.targets.iter().find_map(|bound| {
                if bound.group == group
                    && self.same_target_object(*bound)
                    && let Target::Card(id) = bound.target
                    && self.on_battlefield(id)
                    && self.state.objects[id as usize].card.is_creature()
                {
                    Some(id)
                } else {
                    None
                }
            })
        };
        let (Some(source), Some(recipient)) = (creature(source), creature(recipient)) else {
            return;
        };
        let power = self.stats(source).power.max(0);
        let source_damage = DamageEvent {
            source: self.damage_source(source),
            target: Target::Card(recipient),
            amount: if fight && source == recipient {
                power
                    .checked_mul(2)
                    .expect("damage representation exhausted")
            } else {
                power
            },
            combat: false,
        };
        if !fight || source == recipient {
            self.apply_damage_batch(&[source_damage]);
        } else {
            let return_damage = DamageEvent {
                source: self.damage_source(recipient),
                target: Target::Card(source),
                amount: self.stats(recipient).power.max(0),
                combat: false,
            };
            self.apply_damage_batch(&[source_damage, return_damage]);
        }
    }
}
