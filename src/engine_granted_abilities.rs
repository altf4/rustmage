//! Event-qualified ability grants reuse immutable Equipment programs and owned packets.
use super::*;
impl Game {
    pub(super) fn attachment_recipient(&self, grantor: CardId) -> Option<ObjectRef> {
        let giver = self.state.objects.get(grantor as usize)?;
        let recipient = giver.attached_to?;
        (giver.zone == Zone::Battlefield
            && !self.is_creature(grantor)
            && self.on_battlefield(recipient.card)
            && self.object_ref(recipient.card) == recipient
            && if ability_world::CharacteristicWorld::is_aura(self, grantor) {
                self.aura_can_enchant(giver.card, giver.controller, recipient)
            } else {
                ability_world::CharacteristicWorld::is_equipment(self, grantor)
                    && self.is_creature(recipient.card)
            })
        .then_some(recipient)
    }
    /// Only indexed observers are visited. Ordinary sources allocate no grant packet.
    pub(super) fn indexed_trigger_sources(
        &self,
        ids: &[CardId],
    ) -> Vec<(ObjectView, Option<AttachmentTriggerView>)> {
        let mut out = Vec::new();
        for &id in ids {
            let giver = self.object_view(id);
            if giver.card.triggers().iter().any(|a| {
                !a.condition.is_granted()
                    && matches!(
                        a.condition.event(),
                        TriggerCondition::CreatureCardsEnterYourGraveyard
                            | TriggerCondition::SpellCast { .. }
                            | TriggerCondition::CardDrawn { .. }
                            | TriggerCondition::PlayerDamaged { .. }
                            | TriggerCondition::CreatureDamaged { .. }
                            | TriggerCondition::SelfDamaged
                            | TriggerCondition::AtStep { .. }
                    )
            }) {
                out.push((giver.clone(), None));
            }
            if ability_world::CharacteristicWorld::harnessed(self, id)
                && giver.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::HarnessedTrigger(_)
                        )
                    })
                })
            {
                out.push((
                    giver.clone(),
                    Some(AttachmentTriggerView {
                        grantor: giver.reference,
                        granting_card: giver.card,
                        recipient: giver.reference,
                        defender: None,
                        granted: true,
                        origin: giver.copied_grant_origin(),
                    }),
                ));
            }
            if giver
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.is_granted())
                && let Some(recipient) = self.attachment_recipient(id)
            {
                out.push((
                    self.object_view(recipient.card),
                    Some(AttachmentTriggerView {
                        grantor: giver.reference,
                        granting_card: giver.card,
                        recipient,
                        defender: None,
                        granted: true,
                        origin: giver.copied_grant_origin(),
                    }),
                ));
            }
        }
        out
    }
}
