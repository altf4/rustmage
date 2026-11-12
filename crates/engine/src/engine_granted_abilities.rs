//! Event-qualified ability grants reuse immutable programs and owned packets.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn attachment_recipient(&self, grantor: CardId) -> Option<ObjectRef> {
        let giver = self.state.objects.get(grantor as usize)?;
        let recipient = giver.attached_to?;
        (giver.zone == Zone::Battlefield
            && !self.is_creature(grantor)
            && self.on_battlefield(recipient.card)
            && self.object_ref(recipient.card) == recipient
            && if ability_world::CharacteristicWorld::is_aura(self, grantor) {
                self.battlefield_aura_can_enchant(grantor, recipient)
            } else {
                equipment::can_attach(self, self.object_ref(grantor), recipient)
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
                        TriggerCondition::SelfCrewsOrSaddles
                            | TriggerCondition::ScryOrSurveil
                            | TriggerCondition::Proliferated(_)
                            | TriggerCondition::PermanentCardsEnterYourGraveyard
                            | TriggerCondition::CreatureCardsEnterYourGraveyard
                            | TriggerCondition::SpellCast { .. }
                            | TriggerCondition::SpellCastUsingSourceMana(_)
                            | TriggerCondition::SpellCastOrLandPlayed
                            | TriggerCondition::ModalSpellCastByYou
                            | TriggerCondition::FirstSpellKindOrOtherCreatureType { .. }
                            | TriggerCondition::CardDrawn { .. }
                            | TriggerCondition::LibrarySearched { .. }
                            | TriggerCondition::DiscoveredYou
                            | TriggerCondition::PlayerDamaged { .. }
                            | TriggerCondition::CombatDamageToController
                            | TriggerCondition::CreatureDamaged { .. }
                            | TriggerCondition::ExcessNoncombatDamageToCreatures { .. }
                            | TriggerCondition::SelfDamaged
                            | TriggerCondition::AttachedCreatureDamaged
                            | TriggerCondition::ControlledSpellDamagesSoleCreatureTarget
                            | TriggerCondition::ControlledNoncombatDamageMatchesCreatureToughness
                            | TriggerCondition::AtStep { .. }
                            | TriggerCondition::OwnUntapStepUntaps
                            | TriggerCondition::SelfTurnedFaceUp
                            | TriggerCondition::PermanentYouControlTurnedFaceUp
                            | TriggerCondition::YouTurnPermanentFaceUp
                            | TriggerCondition::YouManifestDread
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

    /// Controlled-creature grants are indexed by their provider, but each
    /// surviving recipient owns a separate trigger. Keep the exact template
    /// slot so a provider's other granted abilities cannot fire from this row.
    pub(super) fn controlled_creature_cast_trigger_sources(
        &self,
        caster: Player,
    ) -> Vec<(ObjectView, AttachmentTriggerView, usize)> {
        let mut sources = Vec::new();
        let query = std::cell::OnceCell::new();
        for &id in &self.state.cast_sources {
            let provider = &self.state.objects[id as usize];
            if provider.zone != Zone::Battlefield {
                continue;
            }
            let grantor = self.object_ref(id);
            let granting_card = provider.card;
            for (index, group) in granting_card.static_ability_groups().iter().enumerate() {
                for fragment in group.fragments {
                    let StaticAbilityFragment::ControlledCreatureTrigger {
                        ability,
                        other,
                        nontoken,
                    } = *fragment
                    else {
                        continue;
                    };
                    let condition = granting_card.triggers()[usize::from(ability)].condition;
                    let TriggerCondition::SpellCast { relation, .. } = condition.event() else {
                        continue;
                    };
                    if !condition.is_granted()
                        || match relation {
                            CastRelation::You => provider.controller != caster,
                            CastRelation::Opponent => provider.controller == caster,
                            CastRelation::Any => false,
                        }
                    {
                        continue;
                    }

                    let world = query.get_or_init(|| self.characteristic_query());
                    if !world.continuous_group_available(grantor, index, group) {
                        continue;
                    }
                    for (recipient, object) in self.state.objects.iter().enumerate() {
                        let recipient = recipient as CardId;
                        if object.zone != Zone::Battlefield
                            || !world.controlled_creature_trigger_applies(
                                grantor, recipient, other, nontoken,
                            )
                        {
                            continue;
                        }
                        let source = world.object_view(recipient);
                        if !source.grant_available(grantor, granting_card, ability, None) {
                            continue;
                        }
                        let grant = AttachmentTriggerView {
                            grantor,
                            granting_card,
                            recipient: source.reference,
                            defender: None,
                            granted: true,
                            origin: self.copied_grant_origin(grantor),
                        };
                        sources.push((source, grant, usize::from(ability)));
                    }
                }
            }
        }
        sources
    }
}
