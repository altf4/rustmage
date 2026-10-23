//! Independent one-cast affinity grants; cost queries are read-only.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NextSpellAffinityView {
    pub source: ObjectRef,
    pub controller: Player,
    pub turn: u32,
}
impl Game {
    pub(super) fn grant_next_spell_affinity(&mut self, item: &SpellView) {
        self.state
            .next_spell_affinities
            .get_or_insert_with(Default::default)
            .push(NextSpellAffinityView {
                source: item.object,
                controller: item.controller,
                turn: self.state.turn,
            });
    }
    pub(super) fn next_noncreature_affinity_reduction(
        &self,
        actor: Player,
        definition: Card,
    ) -> u64 {
        if definition.is_creature() {
            return 0;
        }
        let count = self.state.next_spell_affinities.as_ref().map_or(0, |rows| {
            rows.iter()
                .filter(|row| row.controller == actor && row.turn == self.state.turn)
                .count()
        });
        if count == 0 {
            return 0;
        }
        let artifacts = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == actor
                    && self.current_kinds(*id as CardId) & 16 != 0
            })
            .count();
        (count as u64).saturating_mul(artifacts as u64)
    }
    pub(super) fn consume_next_spell_affinities(&mut self, actor: Player, kinds: u16) {
        if kinds & 1 != 0 {
            return;
        }
        if let Some(rows) = &mut self.state.next_spell_affinities {
            rows.retain(|row| row.controller != actor || row.turn != self.state.turn);
            if rows.is_empty() {
                self.state.next_spell_affinities = None;
            }
        }
    }
}
