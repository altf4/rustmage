//! Effective base-power consumers use the same layer boundary as ordinary stats.
use super::*;

impl Game {
    /// Compute the spell's live keyword, without copying a continuous grant
    /// into its copiable values or ability packet. The spell may currently be
    /// resolving; it remains on the stack until its resolution finishes.
    pub(super) fn spell_has_split_second(&self, spell: &SpellView) -> bool {
        let Some(source) = spell.source else {
            return false;
        };
        spell.kind == StackKind::Spell
            && self
                .state
                .objects
                .get(source.id as usize)
                .is_some_and(|object| {
                    object.zone == Zone::Stack && object.generation == source.object.generation
                })
            && self.keywords(source.id).contains(Keywords::SPLIT_SECOND)
    }

    pub(super) fn spell_with_current_abilities(&self, item: &SpellView) -> SpellView {
        let mut view = item.clone();
        view.split_second = self.spell_has_split_second(item);
        view
    }
    /// Active rule restrictions are sampled live, including during resolution.
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
        if combat
            && self.state.static_sources.iter().copied().any(|id| {
                let source = &self.state.objects[id as usize];
                self.on_battlefield(id)
                    && source.card.static_ability_groups().iter().any(|g| {
                        g.fragments.iter().any(|f| {
                            matches!(
                                f,
                                crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                            )
                        })
                    })
                    && self.continuous_fragment_available(id, |f| {
                        matches!(
                            f,
                            crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                        )
                    })
            })
        {
            return true;
        }
        self.state
            .stack
            .iter()
            .any(|spell| self.spell_has_split_second(spell))
            || self
                .state
                .resolving
                .as_ref()
                .is_some_and(|r| self.spell_has_split_second(&r.item))
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
        let Some(source) = item.source else {
            return;
        };
        // Capture the entire simultaneous batch before any counter replacement
        // or placement can change a later recipient's characteristics.
        let changes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let id = index as CardId;
                if !self.creature_matches(filter, item.controller, source.object, id) {
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
