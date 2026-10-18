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
    pub entry: Option<ObjectRef>,
}

impl SpellView {
    pub(super) fn gift_context(&self) -> Option<GiftContext> {
        if let Some(context) = &self.gift {
            return Some(**context);
        }
        let Some(GiftDeclaration::Promised { beneficiary }) = self.cast?.gift else {
            return None;
        };
        Some(GiftContext {
            source: self.object,
            caster: self.controller,
            beneficiary,
            kind: self.source_card.gift()?.kind,
            entry: None,
        })
    }
}

impl Game {
    pub(super) fn return_targeted_spells_to_hand(&mut self, item: &SpellView) {
        for bound in &item.targets {
            if let Some(victim) = self.bound_stack_item(bound).cloned() {
                // A return is not a counter; uncounterable spells remain valid.
                if victim
                    .cast
                    .is_some_and(|c| matches!(c.face, CastFace::Adventure | CastFace::Omen))
                {
                    self.move_adventure_spell(&victim, Zone::Hand);
                } else {
                    self.move_object(victim.card, Zone::Hand);
                }
            }
        }
    }
    pub(super) fn give_gift(&mut self, item: &SpellView) {
        let Some(context) = item.gift_context() else {
            return;
        };
        match context.kind {
            GiftKind::Card => self.draw_many(context.beneficiary, 1),
            GiftKind::Food | GiftKind::Treasure | GiftKind::TappedFish => {
                let spec = item.source_card.gift().expect("complete Gift recipe");
                self.create_tokens(
                    context.beneficiary,
                    spec.token.expect("Gift token factory"),
                    1,
                    context.kind == GiftKind::TappedFish,
                );
            }
        }
    }
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
        proposal.plan = plan;
        self.targets_available(
            plan,
            proposal.item.controller,
            proposal.item.target_source(),
        ) && self.announcement_target_fundable(&proposal, Some(plan), None, None, None, false)
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
