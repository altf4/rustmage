//! Frozen attacking sets and the exact result of their zone moves.
use super::*;
pub(super) fn moved_creature_count(events: &[ZoneChange]) -> u64 {
    // Count completed affected events, including redirected destinations. Tokens
    // and animated noncreature cards count; prevented non-moves do not.
    u64::try_from(events.iter().filter(|event| event.moved).count())
        .capacity("attacking exile count exhausted")
}
impl Game {
    pub(super) fn exile_attacking_creatures(&mut self, players: PlayerSet, item: &SpellView) {
        let players: Vec<_> = self.program_players(players, item).collect();
        let ids: Vec<_> = self
            .state
            .attackers
            .iter()
            .map(|attacker| attacker.card)
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.is_creature(id)
                    && players.contains(&self.state.objects[id as usize].controller)
            })
            .collect();
        let sources = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.state.resolving.as_mut().unwrap().moved_count = Some(moved_creature_count(&events));
        self.finish_zone_events(&events, &sources);
    }
}
