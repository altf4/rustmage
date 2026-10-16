//! Command-zone emblem objects share immutable definitions and the owned arena.
use super::*;
impl Game {
    pub(super) fn create_emblem(&mut self, controller: Player, factory: Card) {
        assert!(factory.is_emblem());
        let id = CardId::try_from(self.state.objects.len()).capacity("object slots exhausted");
        self.state.objects.push(Object {
            card: factory,
            owner: controller,
            zone: Zone::Command,
            generation: 0,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: Counters::default(),
            used_abilities: 0,
            attached_to: None,
        });
        // The compiler admits only life-gain observers in this emblem family.
        assert!(
            factory
                .triggers()
                .iter()
                .all(|a| matches!(a.condition.event(), TriggerCondition::LifeGained { .. }))
        );
        self.state.life_sources.push(id);
    }
}
