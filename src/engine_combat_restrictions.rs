//! Continuous restrictions are checked when declaring attackers and blockers.
use super::*;
impl Game {
    /// Shared eligibility for declaration candidates and mandatory-attack checks.
    /// Selection does not tap an attacker; costs and tapping happen at commit.
    pub(super) fn can_declare_attacker(&self, actor: Player, id: CardId) -> bool {
        let Some(object) = self.state.objects.get(id as usize) else {
            return false;
        };
        if object.controller != actor
            || object.zone != Zone::Battlefield
            || !self.is_creature(id)
            || object.tapped
        {
            return false;
        }
        let keywords = self.keywords(id);
        (!object.summoning_sick || keywords.contains(Keywords::HASTE))
            && !keywords.contains(Keywords::DEFENDER)
            && !self.combat_restricted(id, true)
    }

    pub(super) fn attacks_each_combat(&self, id: CardId) -> bool {
        self.on_battlefield(id)
            && self.state.objects[id as usize]
                .card
                .static_ability_groups()
                .iter()
                .any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttacksEachCombat
                        )
                    })
                })
            && self.continuous_fragment_available(id, |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::AttacksEachCombat
                )
            })
    }

    /// Every mandatory attacker must attack if at least one legal defender exists.
    /// Defender-specific restrictions do not create a requirement to attack a player.
    pub(super) fn attack_declaration_complete(&self, actor: Player) -> bool {
        let chosen = &self.state.selected_attackers;
        for (index, attacker) in chosen.iter().enumerate() {
            if !self.can_declare_attacker(actor, attacker.card)
                || chosen[..index]
                    .iter()
                    .any(|other| other.card == attacker.card)
                || attacker
                    .defender
                    .is_none_or(|defender| !self.can_attack_defender(attacker.card, defender))
                || !match attacker.defender {
                    Some(CombatDefender::Player(player)) => player == actor.other(),
                    Some(CombatDefender::Planeswalker(reference)) => self
                        .state
                        .objects
                        .get(reference.card as usize)
                        .is_some_and(|object| {
                            object.zone == Zone::Battlefield
                                && object.generation == reference.generation
                                && object.controller == actor.other()
                                && self.current_kinds(reference.card) & 64 != 0
                        }),
                    None => false,
                }
            {
                return false;
            }
        }
        self.state.static_sources.iter().all(|&id| {
            !self.attacks_each_combat(id)
                || !self.can_declare_attacker(actor, id)
                || !self.can_attack_any_defender(actor, id)
                || chosen.iter().any(|attacker| attacker.card == id)
        })
    }

    fn combat_restriction_from(&self, source: CardId, subject: CardId, attack: bool) -> bool {
        let object = &self.state.objects[source as usize];
        let Some(rule) = object.card.combat_restriction() else {
            return false;
        };
        object.zone == Zone::Battlefield
            && self.continuous_fragment_available(source, |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CombatRestriction(..)
                )
            })
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
