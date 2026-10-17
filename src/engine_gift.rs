//! Non-targeted Gift declarations and exact owned cast/entry provenance.
use super::*;
use crate::cards::GiftKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum GiftDeclaration {
    Declined,
    Promised { beneficiary: Player },
}

/// A paid promise belongs to this cast and its actual public entry, not a CardId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GiftContext {
    pub source: ObjectRef,
    pub caster: Player,
    pub beneficiary: Player,
    pub kind: GiftKind,
}

impl Game {
    pub(super) fn gift_choice_fundable(
        &self,
        announcement: &Announcement,
        beneficiary: Option<Player>,
    ) -> bool {
        if beneficiary == Some(announcement.item.controller) {
            return false;
        }
        let mut proposal = announcement.clone();
        proposal.item.cast.as_mut().unwrap().gift = Some(match beneficiary {
            None => GiftDeclaration::Declined,
            Some(beneficiary) => GiftDeclaration::Promised { beneficiary },
        });
        let plan = self.target_spec(&proposal.item);
        self.targets_available(
            plan,
            proposal.item.controller,
            proposal.item.target_source(),
        ) && proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }

    pub(super) fn choose_gift(&mut self, beneficiary: Option<Player>) {
        let announcement = self.state.announcing.as_mut().unwrap();
        announcement.item.cast.as_mut().unwrap().gift = Some(match beneficiary {
            None => GiftDeclaration::Declined,
            Some(beneficiary) => GiftDeclaration::Promised { beneficiary },
        });
        // Declaration precedes targets: select the entire promised/declined plan.
        announcement.plan = announcement
            .item
            .program()
            .map_or(TargetPlan::NONE, |p| p.0);
        announcement.group = 0;
        self.continue_announcement_choices(false);
    }
}
