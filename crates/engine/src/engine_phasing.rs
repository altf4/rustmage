//! CR 702.26: phasing changes a permanent's status, never its incarnation.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Phase out one simultaneous group. Attached permanents phase out
    /// indirectly even when they also match the direct effect.
    pub(super) fn phase_out_batch(&mut self, direct: &[CardId]) {
        if direct.is_empty() {
            return;
        }
        let len = self.state.objects.len();
        let mut affected = vec![false; len];
        let mut indirect = vec![false; len];
        for &id in direct {
            if self.on_battlefield(id) {
                affected[id as usize] = true;
            }
        }
        // Attachment chains are short; this only scans after a phasing event.
        let mut changed = true;
        while changed {
            changed = false;
            for (index, object) in self.state.objects.iter().enumerate() {
                if object.zone != Zone::Battlefield {
                    continue;
                }
                let Some(host) = object.attached_to else {
                    continue;
                };
                if !affected.get(host.card as usize).copied().unwrap_or(false)
                    || self.object_ref(host.card) != host
                {
                    continue;
                }
                if !affected[index] {
                    affected[index] = true;
                    changed = true;
                }
                indirect[index] = true;
            }
        }
        if !affected.iter().any(|&value| value) {
            return;
        }
        let phased_defenders: Vec<_> = affected
            .iter()
            .enumerate()
            .filter_map(|(id, &phased)| phased.then(|| self.object_ref(id as CardId)))
            .collect();
        for (index, object) in self.state.objects.iter_mut().enumerate() {
            if affected[index] {
                object.zone = Zone::PhasedOut;
                object.phased_out_under = Some(object.controller);
                object.phased_out_indirectly = indirect[index];
            }
        }
        // The same creatures cease to be combatants without leaving the
        // battlefield or firing any departure/attachment triggers.
        self.state
            .attackers
            .retain(|row| !affected[row.card as usize]);
        self.state.blocking.retain(|&id| !affected[id as usize]);
        self.state.blocks.retain(|&(blocker, attacker)| {
            !affected[blocker as usize] && !affected[attacker as usize]
        });
        self.state
            .selected_attackers
            .retain(|row| !affected[row.card as usize]);
        for attack in self
            .state
            .attackers
            .iter_mut()
            .chain(&mut self.state.selected_attackers)
        {
            if matches!(attack.defender, Some(CombatDefender::Planeswalker(r) | CombatDefender::Battle(r)) if phased_defenders.contains(&r))
            {
                attack.defender = None;
            }
        }
        self.state.selected_blocks.retain(|&(blocker, attacker)| {
            !affected[blocker as usize] && !affected[attacker as usize]
        });
        self.state
            .first_strikers
            .retain(|&id| !affected[id as usize]);
    }

    /// A typed mass effect such as The City's chapter I uses live types at
    /// resolution and phases the selected permanents simultaneously.
    pub(super) fn phase_out_all(&mut self, kinds: u16) {
        let direct: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| self.on_battlefield(id) && self.current_kinds(id) & kinds != 0)
            .collect();
        self.phase_out_batch(&direct);
    }

    fn phase_in_prohibited(&self) -> bool {
        self.state.static_sources.iter().copied().any(|id| {
            self.on_battlefield(id)
                && self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(fragment, StaticAbilityFragment::PermanentsCantPhaseIn)
                        })
                    })
                && self.continuous_fragment_available(id, |fragment| {
                    matches!(fragment, StaticAbilityFragment::PermanentsCantPhaseIn)
                })
        })
    }

    /// CR 502.1: run before that player's ordinary untaps. A phased-out
    /// attachment returns with its host, independent of its own controller.
    pub(super) fn phase_in_for_untap(&mut self, active: Player) {
        // Ordinary games never allocate phasing scratch state at turn start.
        if !self
            .state
            .objects
            .iter()
            .any(|object| object.zone == Zone::PhasedOut)
        {
            return;
        }
        if self.phase_in_prohibited() {
            return;
        }
        let len = self.state.objects.len();
        let mut returning = vec![false; len];
        for (id, object) in self.state.objects.iter().enumerate() {
            returning[id] = object.zone == Zone::PhasedOut
                && !object.phased_out_indirectly
                && object.phased_out_under == Some(active);
        }
        let mut changed = true;
        while changed {
            changed = false;
            for (id, object) in self.state.objects.iter().enumerate() {
                if object.zone != Zone::PhasedOut || !object.phased_out_indirectly || returning[id]
                {
                    continue;
                }
                let Some(host) = object.attached_to else {
                    continue;
                };
                if returning.get(host.card as usize).copied().unwrap_or(false)
                    && self.object_ref(host.card) == host
                {
                    returning[id] = true;
                    changed = true;
                }
            }
        }
        let stale_direct_attachments: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                if !returning[id] || object.phased_out_indirectly {
                    return None;
                }
                let host = object.attached_to?;
                let same_battlefield_incarnation = self
                    .state
                    .objects
                    .get(host.card as usize)
                    .is_some_and(|recipient| {
                        matches!(recipient.zone, Zone::Battlefield | Zone::PhasedOut)
                            && recipient.generation == host.generation
                    });
                (!same_battlefield_incarnation).then(|| self.object_ref(id as CardId))
            })
            .collect();
        for (id, object) in self.state.objects.iter_mut().enumerate() {
            if returning[id] {
                object.zone = Zone::Battlefield;
                object.phased_out_under = None;
                object.phased_out_indirectly = false;
                if object.controller == active {
                    object.summoning_sick = false;
                }
            }
        }
        // CR 702.26i/j: a directly phased attachment whose old host left the
        // battlefield returns unattached, without an attachment event.
        for attachment in stale_direct_attachments {
            self.unattach(attachment);
        }
        if returning.iter().any(|&value| value) {
            // Duration-based control effects may have expired while their
            // recipient was treated as nonexistent.
            self.refresh_controllers_after(None);
        }
    }
}
