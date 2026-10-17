//! Continuous restrictions are checked when declaring attackers and blockers.
use super::*;
impl Game {
    fn combat_restriction_from(&self, source: CardId, subject: CardId, attack: bool) -> bool {
        let object = &self.state.objects[source as usize];
        let Some(rule) = object.card.combat_restriction() else {
            return false;
        };
        object.zone == Zone::Battlefield
            && if attack { rule.attack } else { rule.block }
            && match rule.subject {
                Subject::Source => source == subject,
                Subject::Attached => object.attached_to == Some(self.object_ref(subject)),
                _ => false,
            }
            && rule.unless.is_none_or(|guard| {
                !self.step_guard(
                    guard,
                    object.controller,
                    self.object_ref(source),
                    object.tapped,
                )
            })
    }
    pub(super) fn combat_restricted(&self, id: CardId, attack: bool) -> bool {
        self.combat_restriction_from(id, id, attack)
            || self
                .state
                .attachment_sources
                .iter()
                .any(|&source| self.combat_restriction_from(source, id, attack))
    }
}
