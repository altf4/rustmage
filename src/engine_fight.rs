//! Noncombat damage using current creature characteristics and distinct operands.
use super::*;
impl Game {
    pub(super) fn reflexive_fight(&mut self, item: &SpellView) {
        let Some(context) = &item.reflexive_subject else {
            return;
        };
        // Only the opponent is targeted. Construct operands after target validation;
        // the original creature is an exact captured reference, never a new target.
        let mut operands = item.clone();
        for target in &mut operands.targets {
            target.group = 1;
        }
        operands.targets.push(BoundTarget {
            target: Target::Card(context.object.card),
            object: Some(context.object),
            group: 0,
        });
        self.creature_power_damage(&operands, 0, 1, true);
    }
    pub(super) fn creature_power_damage(
        &mut self,
        item: &SpellView,
        source: u16,
        recipient: u16,
        fight: bool,
    ) {
        let permanent = |group: u16, creature_only: bool| {
            let kinds = if creature_only { 1 } else { 1 | 64 };
            item.targets.iter().find_map(|bound| {
                if bound.group == group
                    && self.same_target_object(*bound)
                    && let Target::Card(id) = bound.target
                    && self.on_battlefield(id)
                    && self.current_kinds(id) & kinds != 0
                {
                    Some(id)
                } else {
                    None
                }
            })
        };
        let (Some(source), Some(recipient)) =
            (permanent(source, true), permanent(recipient, fight))
        else {
            return;
        };
        let power = self.stats(source).power.max(0);
        let source_damage = DamageEvent {
            source: self.damage_source(source),
            target: Target::Card(recipient),
            amount: if fight && source == recipient {
                power
                    .checked_mul(2)
                    .capacity("damage representation exhausted")
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
