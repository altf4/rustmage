//! Untap prohibitions are external continuous rules, distinct from abilities on
//! the affected permanent. Prohibitions apply before stun replacements.
use super::*;
use crate::cards::{StaticAbilityFragment, UntapRestriction};

#[derive(Clone, Copy)]
pub(super) enum UntapCause {
    Effect,
    ControllerStep(Player),
}
impl Game {
    fn has_untap_rules(&self) -> bool {
        if self.state.attachment_sources.iter().any(|&id| {
            self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.event() == TriggerCondition::AttachedBecomesUntapped)
        }) {
            return true;
        }
        self.state
            .static_sources
            .iter()
            .chain(&self.state.attachment_sources)
            .any(|&id| {
                self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                StaticAbilityFragment::AttachedUntap(_)
                                    | StaticAbilityFragment::SelfUntap(_)
                            )
                        })
                    })
            })
    }
    pub(super) fn untap_allowed(&self, id: CardId, cause: UntapCause) -> bool {
        let recipient = self.object_ref(id);
        let evaluation = self.evaluated_abilities();
        let object = &self.state.objects[id as usize];
        for (index, group) in object.card.static_ability_groups().iter().enumerate() {
            if evaluation
                .as_ref()
                .is_some_and(|evaluation| !evaluation.group_available(recipient, index))
            {
                continue;
            }
            for fragment in group.fragments {
                if matches!(
                    fragment,
                    StaticAbilityFragment::SelfUntap(UntapRestriction::All)
                ) || matches!(
                    fragment,
                    StaticAbilityFragment::SelfUntap(UntapRestriction::ControllerStep)
                ) && matches!(cause, UntapCause::ControllerStep(player) if player == object.controller)
                {
                    return false;
                }
            }
        }
        for &source in &self.state.attachment_sources {
            let object = &self.state.objects[source as usize];
            if object.zone != Zone::Battlefield || object.attached_to != Some(recipient) {
                continue;
            }
            for (index, group) in object.card.static_ability_groups().iter().enumerate() {
                if evaluation.as_ref().is_some_and(|evaluation| {
                    !evaluation.group_available(self.object_ref(source), index)
                }) {
                    continue;
                }
                for fragment in group.fragments {
                    match fragment {
                        StaticAbilityFragment::AttachedUntap(UntapRestriction::All) => {
                            return false;
                        }
                        StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep)
                            if matches!(cause, UntapCause::ControllerStep(player)
                                if player == self.state.objects[id as usize].controller) =>
                        {
                            return false;
                        }
                        _ => (),
                    }
                }
            }
        }
        true
    }
    pub(super) fn untap_objects(&mut self, ids: &[CardId], cause: UntapCause) {
        if !self.has_untap_rules() {
            for &id in ids {
                if self.on_battlefield(id) {
                    self.state.objects[id as usize].untap();
                }
            }
            return;
        }
        // Determine every member before any stun counter is removed. Untapping
        // is simultaneous, including when an effect affects several permanents.
        let allowed: Vec<_> = ids
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id) && self.untap_allowed(id, cause))
            .collect();
        let before: Vec<_> = allowed
            .iter()
            .copied()
            .filter(|&id| self.state.objects[id as usize].tapped)
            .map(|id| self.object_ref(id))
            .collect();
        for id in allowed {
            self.state.objects[id as usize].untap();
        }
        let untapped: Vec<_> = before
            .into_iter()
            .filter(|r| !self.state.objects[r.card as usize].tapped)
            .collect();
        self.attached_untap_triggers(&untapped);
    }
    pub(super) fn untap_active_player(&mut self) {
        if !self.has_untap_rules() {
            for object in &mut self.state.objects {
                if object.zone == Zone::Battlefield && object.controller == self.state.active {
                    object.untap();
                }
            }
            return;
        }
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.controller == self.state.active)
                    .then_some(id as CardId)
            })
            .collect();
        self.untap_objects(&ids, UntapCause::ControllerStep(self.state.active));
    }
}
