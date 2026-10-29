//! Effective base-power consumers use the same layer boundary as ordinary stats.
use super::*;

impl Game {
    /// Active rule restrictions are sampled live, including while a spell is being cast.
    pub(super) fn nonmana_actions_prohibited(&self) -> bool {
        let combat = matches!(
            self.state.step,
            Step::BeginCombat
                | Step::Attackers
                | Step::Blockers
                | Step::FirstStrikeDamage
                | Step::CombatDamage
                | Step::EndCombat
        );
        let providers = self
            .state
            .static_sources
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id));
        for id in providers {
            let source = &self.state.objects[id as usize];
            if combat
                && self.continuous_fragment_available(id, |f| {
                    matches!(
                        f,
                        crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                    )
                })
                && source.card.static_ability_groups().iter().any(|g| {
                    g.fragments.iter().any(|f| {
                        matches!(
                            f,
                            crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                        )
                    })
                })
            {
                return true;
            }
            if source.card.static_ability_groups().iter().any(|g| {
                g.fragments.iter().any(|f| {
                    matches!(
                        f,
                        crate::cards::StaticAbilityFragment::ControlledInstantSorcerySplitSecond
                    )
                })
            }) && self.continuous_fragment_available(id, |f| {
                matches!(
                    f,
                    crate::cards::StaticAbilityFragment::ControlledInstantSorcerySplitSecond
                )
            }) && self.state.stack.iter().any(|item| {
                item.kind == StackKind::Spell
                    && item.controller == source.controller
                    && item.source_facts.kinds & (4 | 8) != 0
            }) {
                return true;
            }
        }
        false
    }
    pub(super) fn static_casting_prohibited(&self, actor: Player) -> bool {
        self.nonmana_actions_prohibited()
            || self.state.static_sources.iter().any(|&id| {
                let object = &self.state.objects[id as usize];
                object.zone == Zone::Battlefield
                    && self.printed_abilities_available(id)
                    && object.controller == self.state.active
                    && object.controller != actor
                    && object.card.static_cast_restriction() & 1 != 0
            })
    }
    pub(super) fn put_power_excess_counters(&mut self, item: &SpellView, filter: CreatureFilter) {
        // Capture the entire simultaneous batch before any counter replacement
        // or placement can change a later recipient's characteristics.
        let changes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let id = index as CardId;
                if !self.creature_matches(filter, item.controller, item.object, id) {
                    return None;
                }
                let (base, current) = self.layered_stats_without_hand(id, None);
                if current.power <= base.power {
                    return None;
                }
                let amount = current
                    .power
                    .checked_sub(base.power)
                    .capacity("power difference exhausted");
                let amount = u64::try_from(amount).capacity("counter amount exhausted");
                let counters = Counters {
                    plus_one: amount,
                    ..Counters::default()
                };
                Some((self.object_ref(id), counters))
            })
            .collect();
        self.effect_counters(changes, replacements::ResumeMode::Program);
    }
}
