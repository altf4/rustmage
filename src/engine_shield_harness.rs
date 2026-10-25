//! Counter-created protection and noncopiable harness designation helpers.
use super::*;
impl Game {
    pub(super) fn remove_shield(&mut self, id: CardId) {
        let mut counters = self.counter_amounts(id).to_owned();
        counters.remove(CounterKind::Shield, 1);
        self.set_counters(id, counters);
    }
    pub(super) fn effect_destruction_prevented(&mut self, id: CardId) -> bool {
        if !self.on_battlefield(id) || self.keywords(id).contains(Keywords::INDESTRUCTIBLE) {
            return true;
        }
        if self.counter_count(id, CounterKind::Shield) > 0 {
            self.remove_shield(id);
            return true;
        }
        false
    }
}
