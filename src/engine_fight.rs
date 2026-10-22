//! Noncombat damage using current creature characteristics and distinct operands.
use super::*;
impl Game {
    pub(super) fn entering_power_damage_players(&mut self, item: &SpellView, players: PlayerSet) {
        let Some(object) = item.entering_object else {
            return;
        };
        let source = if self.on_battlefield(object.card) && self.object_ref(object.card) == object {
            self.damage_source(object.card)
        } else {
            let Some(source) = self
                .state
                .roles
                .as_ref()
                .and_then(|state| {
                    state
                        .stat_subjects
                        .iter()
                        .find(|fact| fact.object == object)
                })
                .and_then(|fact| fact.departure_damage.as_deref())
            else {
                return;
            };
            *source
        };
        // A noncreature permanent has no power characteristic (CR208.3).
        let amount = if source.facts.kinds & 1 != 0 {
            source.facts.power.max(0)
        } else {
            0
        };
        let events: Vec<_> = self
            .program_players(players, item)
            .map(|player| DamageEvent {
                preventable: true,
                source,
                target: Target::Player(player),
                amount,
                combat: false,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
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
        self.creatures_deal_power_damage(source, recipient, fight);
    }
    pub(super) fn attached_fight(&mut self, item: &SpellView) {
        let Some(source) = self.attached_subject(item) else {
            return;
        };
        let target = item.targets.iter().find_map(|bound| {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
                && self.is_creature(id)
            {
                Some(id)
            } else {
                None
            }
        });
        if let Some(target) = target {
            self.creatures_deal_power_damage(source, target, true);
        }
    }
    pub(super) fn source_fight(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card)
            || self.object_ref(item.card) != item.object
            || self.current_kinds(item.card) & 1 == 0
        {
            return;
        }
        let target = item.targets.iter().find_map(|bound| {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
                && self.current_kinds(id) & 1 != 0
            {
                Some(id)
            } else {
                None
            }
        });
        if let Some(target) = target {
            self.creatures_deal_power_damage(item.card, target, true);
        }
    }
    fn creatures_deal_power_damage(&mut self, source: CardId, recipient: CardId, fight: bool) {
        let power = self.stats(source).power.max(0);
        let source_damage = DamageEvent {
            preventable: true,
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
                preventable: true,
                source: self.damage_source(recipient),
                target: Target::Card(source),
                amount: self.stats(recipient).power.max(0),
                combat: false,
            };
            self.apply_damage_batch(&[source_damage, return_damage]);
        }
    }
}
