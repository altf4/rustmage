//! Resolution-owned parity choice and one frozen, untargeted destruction batch.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManaValueParity {
    Odd,
    Even,
}
impl ManaValueParity {
    pub(super) fn matches(self, value: u64) -> bool {
        (value % 2 == 1) == (self == Self::Odd)
    }
}
impl Game {
    pub(super) fn choose_mana_value_parity(&mut self, parity: ManaValueParity) {
        let item = self.state.resolving.as_ref().unwrap().item.clone();
        // The source need not remain. Exclude only its captured incarnation,
        // then freeze eligibility and indestructibility before any departure.
        let mut ids: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.object_ref(id) != item.object
                    && self.is_creature(id)
                    && parity.matches(
                        self.copiable_recipe(self.object_ref(id))
                            .definition
                            .fixed_mana_value(),
                    )
                    && !self.keywords(id).contains(Keywords::INDESTRUCTIBLE)
            })
            .collect();
        ids.retain(|&id| !self.effect_destruction_prevented(id));
        let sources = if ids.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.state.resolving.as_mut().unwrap().moved_count = Some(
            u64::try_from(events.iter().filter(|event| event.moved).count())
                .capacity("parity destruction count exhausted"),
        );
        self.finish_zone_events(&events, &sources);
        self.run_program();
    }
}
