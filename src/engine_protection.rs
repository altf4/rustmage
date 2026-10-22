//! Color protection reuses ordered keyword grants and ordinary prevention choices.
use super::*;
impl Game {
    pub(super) fn protected_from_source(&self, id: CardId, colors: u8, controller: Player) -> bool {
        let keywords = self.keywords(id);
        keywords.contains(Keywords::PROTECTION_EVERYTHING)
            || keywords.contains(Keywords::PROTECTION_OPPONENTS)
                && self.state.objects[id as usize].controller != controller
            || colors != 0 && keywords.contains(Keywords::PROTECTION_COLORS)
    }
    pub(super) fn temporary_player_or_walker_damage_triggers(&mut self, events: &[DamageEvent]) {
        let grants: Vec<_> = self
            .state
            .temporary_abilities
            .as_ref()
            .map_or(&[][..], |rows| rows.as_slice())
            .iter()
            .copied()
            .filter(|grant| {
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        == TriggerCondition::SourceCombatDamagePlayerOrPlaneswalker
            })
            .collect();
        for grant in grants {
            let source = self.object_view(grant.recipient.card);
            if !source.grant_available(
                grant.grantor,
                grant.granting_card,
                grant.ability,
                Some(grant.timestamp),
            ) {
                continue;
            }
            let hits: std::collections::BTreeSet<_> = events
                .iter()
                .filter(|event| {
                    event.combat
                        && event.amount > 0
                        && event.source.reference == grant.recipient
                        && match event.target {
                            Target::Player(_) => true,
                            Target::Card(id) => {
                                self.on_battlefield(id) && self.current_kinds(id) & 64 != 0
                            }
                            _ => false,
                        }
                })
                .map(|event| event.target)
                .collect();
            for _ in hits {
                self.queue_trigger_from(
                    &source,
                    usize::from(grant.ability),
                    Some(Box::new(AttachmentTriggerView {
                        grantor: grant.grantor,
                        granting_card: grant.granting_card,
                        recipient: grant.recipient,
                        defender: None,
                        granted: true,
                    })),
                );
            }
        }
    }
}
