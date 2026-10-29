//! Command-zone emblem objects share immutable definitions and the owned arena.
use super::*;
impl Game {
    pub(super) fn create_emblem(&mut self, controller: Player, factory: Card) {
        assert!(factory.is_emblem());
        let id = CardId::try_from(self.state.objects.len()).capacity("object slots exhausted");
        self.state.objects.push(Object {
            copy_kind: CopyKind::None,
            card: factory,
            owner: controller,
            controller,
            zone: Zone::Command,
            generation: 0,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            cast_for_warp: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            dealt_combat_damage_this_incarnation: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: InitialCounters::default(),
            used_abilities: 0,
            loyalty_activated_this_turn: false,
            class_level: 1,
            attached_to: None,
        });
        self.register_battlefield_sources(id);
    }
}
