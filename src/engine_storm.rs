//! Storm belongs to the actual cast spell and counts every player's earlier spells.
use super::*;
use crate::cards::{CastFilter, StaticAbilityFragment};
impl Card {
    pub(super) fn has_storm_grant(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::GrantInstantSorceryStorm { .. }))
        })
    }
}
impl Game {
    pub(super) fn queue_instant_sorcery_storm(&mut self, index: usize) {
        let spell = &self.state.stack[index];
        if spell.is_copy || !spell.source.is_some_and(|source| CastFilter::InstantSorcery.matches(source.card)) {
            return;
        }
        let spell = spell.clone();
        let prior = self
            .state
            .players
            .iter()
            .try_fold(0u64, |sum, p| sum.checked_add(p.spells_cast_this_turn))
            .capacity("storm count exhausted")
            .checked_sub(1)
            .expect("actual cast counted");
        let mut grants = Vec::new();
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            if source.zone != Zone::Battlefield
                || source.controller != spell.controller
                || !source.card.has_storm_grant()
                || !self.printed_abilities_available(id)
            {
                continue;
            }
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    if let StaticAbilityFragment::GrantInstantSorceryStorm { ability } = *fragment {
                        grants.push((
                            AttachmentTriggerView {
                                grantor: self.object_ref(id),
                                granting_card: source.card,
                                recipient: spell.source.expect("cast spell has source").object,
                                defender: None,
                                granted: true,
                                origin: self.copied_grant_origin(self.object_ref(id)),
                            },
                            ability,
                        ));
                    }
                }
            }
        }
        for (grant, ability) in grants {
            self.queue_granted_spell_trigger(&spell, grant, ability);
            self.state.pending_triggers.last_mut().unwrap().copy_event =
                Some(Box::new(CopyEvent {
                    spell: spell.clone(),
                    amount: prior,
                }));
        }
    }
}
