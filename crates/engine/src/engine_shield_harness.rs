//! Counter-created protection and noncopiable harness designation helpers.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
        let shield = self.counter_count(id, CounterKind::Shield) > 0;
        let surgeon = self.surgeon_replaces_destruction(id);
        if !shield && !surgeon {
            return false;
        }
        let use_surgeon = surgeon && (!shield || self.choose_destruction_replacement(id));
        if use_surgeon {
            self.replace_destruction_with_healing(id);
        } else {
            self.remove_shield(id);
        }
        true
    }

    /// Shield counters replace effect-caused destruction, but not the lethal
    /// damage or deathtouch state-based actions (CR 122.1c, 704.5g-h).
    pub(super) fn sba_destruction_prevented(&mut self, id: CardId) -> bool {
        if !self.surgeon_replaces_destruction(id) {
            return false;
        }
        self.replace_destruction_with_healing(id);
        true
    }

    fn replace_destruction_with_healing(&mut self, id: CardId) {
        let mut counters = self.counter_amounts(id).to_owned();
        let amount = counters.get(CounterKind::PlusOne);
        counters.remove(CounterKind::PlusOne, amount);
        self.set_counters(id, counters);
        let object = &mut self.state.objects[id as usize];
        object.damage = 0;
        object.deathtouch_since_sba = false;
    }

    fn surgeon_replaces_destruction(&self, id: CardId) -> bool {
        if !self.is_creature(id) || self.counter_count(id, CounterKind::PlusOne) == 0 {
            return false;
        }
        let controller = self.state.objects[id as usize].controller;
        let world = self.characteristic_query();
        CharacteristicWorld::static_source_candidates(&world)
            .into_iter()
            .any(|source| {
                source != id
                    && self.on_battlefield(source)
                    && self.state.objects[source as usize].controller == controller
                    && CharacteristicWorld::recipe(&world, source)
                        .definition
                        .static_ability_groups()
                        .iter()
                        .any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    StaticAbilityFragment::SaveOtherPlusOneCreatureFromDestruction
                                )
                            })
                        })
                    && self.continuous_fragment_available(source, |fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::SaveOtherPlusOneCreatureFromDestruction
                        )
                    })
            })
    }
}
